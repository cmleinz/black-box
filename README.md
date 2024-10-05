# Tinsel

A minimal, actor framework inspired by actix, and built around the convenience 
of `async fn` in traits:

```rust, ignore
use tinsel::*;

struct Event;

struct Shutdown;

struct MyActor;

impl Actor for MyActor {}

impl Handler<Event> for MyActor {
    async fn handle(&mut self, msg: Event, _ctx: &Context) {
        println!("DEBUG: New event {}", stringify!(msg));
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

Tinsel is not--and does not ship with--an executor. This means you are required
to bring your own, this means you are free to choose any runtime you would like.

## Message Trait

Tinsel does have a message trait, but currently it is just a supertrait for 
`'static + Send`. This makes it super easy to define new message types, no need
to implement a trait on it, just implement the `Handler` for it, and you're good
to go.
