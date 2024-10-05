# Tinsel

A minimal, actor framework inspired by actix, and built around the convenience 
of `async fn` in traits:

```rust
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
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        println!("INFO: Shut down");
    }
}

#[tokio::main]
async fn main() {
    let actor = MyActor;
    let (executor, address) = Executor::new(actor);

    tokio::spawn(async move {
        for _ in 0..5 {
            address.send(Event).await;
            tokio::time::sleep(std::time::Duration::from_millis(500)).await
        }
        address.send(Shutdown).await;
    });

    executor.run().await;
}
```

## About

Tinsel is not--and does not ship with--an executor. This means you are free to
bring your own.
