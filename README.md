# Black-Box

[![Crates.io](https://img.shields.io/crates/v/black-box.svg)](https://crates.io/crates/black-box)
[![Documentation](https://docs.rs/black-box/badge.svg)](https://docs.rs/black-box/)

A [minimal, stage](https://en.wikipedia.org/wiki/Black_box_theater) for actors.

API design is inspired by actix, but built around the convenience of `async fn`
in traits.

To get started just define an Actor and implement some message handlers:

```rust, no_run
use black_box::*;

struct Event;

struct Shutdown;

struct MyActor;

// All methods are provided, can be overridden for more control
impl Actor for MyActor {}

impl Handler<Event> for MyActor {
    async fn handle(&mut self, msg: Event, _ctx: &Context<Self>) {
        println!("DEBUG: New event {}", stringify!(msg));
    }
}

impl Handler<Shutdown> for MyActor {
    async fn handle(&mut self, _msg: Shutdown, ctx: &Context<Self>) {
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

## Runtime

Black-box deliberatly does not ship with a runtime, instead it aims to work with
the users runtime of choice.

## Send Bounds

While it likely won't always be the case, currently the futures return by 
`Handler::handle` must be `Send`

## Message Trait

Black-box does have a message trait, but currently it is just a supertrait for 
`'static + Send`. This makes it super easy to define new message types, no need
to implement a trait on it, just implement the `Handler` for it, and you're good
to go.

In the future, this might change to accomodate things like message responses, 
however for now the same things can be accomplished by embedding a oneshot
channel in the message type to handle the response.
