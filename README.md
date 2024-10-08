# Black-Box

A [minimal, stage](https://en.wikipedia.org/wiki/Black_box_theater) for actors.

API design is inspired by actix, but built around the convenience of `async fn`
in traits.

To get started just define an Actor and implement some message handlers:

```rust, ignore
use black_box::*;

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

Black-box is not--and does not ship with--an executor. This means you are required
to bring your own, this means you are free to choose any runtime you would like.

## Message Trait

Black-box does have a message trait, but currently it is just a supertrait for 
`'static + Send`. This makes it super easy to define new message types, no need
to implement a trait on it, just implement the `Handler` for it, and you're good
to go.

In the future, this might change to accomodate things like message responses, 
however for now the same things can be accomplished by embedding a oneshot
channel in the message type to handle the response.
