# Black-Box

[![Crates.io](https://img.shields.io/crates/v/black-box.svg)](https://crates.io/crates/black-box)
[![Documentation](https://docs.rs/black-box/badge.svg)](https://docs.rs/black-box/)

A [minimal, stage](https://en.wikipedia.org/wiki/Black_box_theater) for actors.

## About

Black-box's API design is inspired by actix, but built is built to be as minimal
as possible, and to integrate with the recently stabalized `async fn` in traits.

To get started just define an Actor and implement some message handlers:

```rust, no_run
use black_box::*;

// Messages do not need to implement anything
struct Event;

struct Shutdown;

struct MyActor;

// All methods are provided, but can be overridden for more control
impl Actor for MyActor {}

impl Handler<Event> for MyActor {
    async fn handle(&mut self, msg: Event, _ctx: &Context<Self>) {
        println!("DEBUG: New event {}", stringify!(msg));
    }
}

impl Handler<Shutdown> for MyActor {
    async fn handle(&mut self, _msg: Shutdown, ctx: &Context<Self>) {
        println!("INFO: Shutting down");
        // shutdown the actor from within the handler
        ctx.shutdown(); 
		// await futures inline thanks to async fn 
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
the user's runtime of choice.

## Send Bounds

While it likely won't always be the case, currently the futures return by 
`Handler::handle` must be `Send`.

## Message Trait

Black-box does have a message trait, but currently private, and just a
supertrait for `'static + Send`. 

This means messages are not special types, any `'static + Send` type can
immediately be used as a message.

In the future, this might change to accomodate things like message responses, 
however for now the same things can be accomplished by embedding a oneshot
channel in the message type to handle the response.

```rust, ignore
type Message = (String, tokio::sync::oneshot::Sender<String>);

impl Handler<Message> for MyActor {
    async fn handle(&mut self, mut message: Message, _ctx: &Context<Self>) {
        message.0.push_str("foo");
        let _ = message.1.send(message.0);
    }
}
```

## Limitations

Before adopting black-box, there are a few drawbacks to consider.

### Concurrency

The lack of a runtime and the decision to provide `&mut self` in `Handler` means
that **there is no native concurrency within actors in black-box**. Awaiting a
future within `Handler::handle` will not prevent messages from enqueuing, but it
will prevent the processing of those events.

For many applications this will be fine. However, for applications which require
high concurrency, you will likely want to either 

1. Spawn many actors to handle the work
1. Offload the bulk of the asynchronous work to a task spawned onto some
executor.

### Allocations

An implementation of the actor pattern which is based purely around channels
would likely either:

1. Create many channels over which to send different message types
1. Collect all the message types for a given actor into an enum and descructure
the enum, and pass it down to functions

To get around these rough spots, black-box conducts type-erasure via 
`Box<dyn Any>`, when the message is sent, then reconstructs the message type 
before passing it to the appropriate message handler.

This unfortunately results in two allocation for every message, one for this,
and one for the handle future.
