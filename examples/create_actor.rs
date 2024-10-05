use std::time::Duration;

use tinsel::*;

#[derive(Debug)]
struct Event;

impl Message for Event {}

#[derive(Debug)]
struct Shutdown;

impl Message for Shutdown {}

struct MyActor;

impl Actor for MyActor {}

impl Handler<Event> for MyActor {
    async fn handle(&mut self, msg: Event, _ctx: &Context) {
        println!("DEBUG: New event {:?}", msg);
    }
}

impl Handler<Shutdown> for MyActor {
    async fn handle(&mut self, _msg: Shutdown, ctx: &Context) {
        println!("INFO: Shutting down");
        ctx.shutdown();
    }
}

fn main() {
    let ex = async_executor::Executor::new();

    let actor = MyActor;
    let (executor, address) = Executor::new(actor);

    std::thread::spawn(move || {
        for _ in 0..5 {
            address.try_send(Event);
            std::thread::sleep(Duration::from_millis(500));
        }
        address.try_send(Shutdown);
    });
    let task = ex.spawn(async move { executor.run().await });

    futures_lite::future::block_on(ex.run(task));
}
