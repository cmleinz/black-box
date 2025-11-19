use std::future::Future;

use async_channel::{Receiver, Sender};

use crate::{
    Actor, Address, WeakAddress,
    error::{ActorError, AddressError},
    message::Envelope,
};

const DEFAULT_CAP: usize = 100;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum State {
    #[default]
    Continue,
    Shutdown,
    SendersClosed,
}

/// A cloneable context for the actor.
///
/// Currently this fuctions as a means by which to alter the state of the [`Executor`], it is
/// cloneable and can thus be sent to other threads, runtimes or even other actors to trigger a
/// shutdown.
#[derive(Debug, Clone)]
pub struct Context<A> {
    sender: async_channel::Sender<State>,
    address: WeakAddress<A>,
}

impl<A> Context<A> {
    /// Triggers the end of the executor.
    ///
    /// Once triggered, no new messages will be processed and the actor will exit after resolving
    /// [`Actor::stopping`]
    pub fn shutdown(&self) {
        let _ = self.sender.force_send(State::Shutdown);
    }

    /// Retrieve the address for the executor's actor
    ///
    /// This is useful when an actor wants to emit messages to itself.
    pub fn address(&self) -> &WeakAddress<A> {
        &self.address
    }
}

/// The event loop for an actor
///
/// Handles the receipt of messages, and state management of the actor. The primary method exposed
/// by the executor is [`Executor::run`], which is used to execute the event loop.
///
/// # Example
///
/// A common pattern is to spawn the executor onto an async runtime like tokio.
///
/// ```no_run
/// # use black_box::*;
/// # struct MyActor;
/// # impl Actor for MyActor {}
/// let my_actor = MyActor;
/// let (mut executor, addr) = Executor::new(my_actor);
///
/// tokio::spawn(async move { executor.run().await });
/// ```
#[derive(Debug)]
pub struct Executor<A> {
    actor: A,
    context: Context<A>,
    state: State,
    from_context: Receiver<State>,
    receiver: Receiver<Envelope<A>>,
}

#[derive(Debug, Clone)]
pub struct ShutdownHandle(Sender<State>);

impl ShutdownHandle {
    pub fn shutdown(&self) -> Result<(), ActorError> {
        self.0
            .force_send(State::Shutdown)
            .map(|_| ())
            .map_err(|_| ActorError::Shutdown)
    }
}

impl<A> Executor<A> {
    pub fn new(actor: A) -> (Self, Address<A>) {
        Self::new_with_capacity(actor, DEFAULT_CAP)
    }

    pub fn new_with_capacity(actor: A, cap: usize) -> (Self, Address<A>) {
        let (sender, receiver) = async_channel::bounded(cap);
        let address = Address::new(sender);
        let (state_tx, state_rx) = async_channel::unbounded();
        let me = Self {
            actor,
            receiver,
            context: Context {
                sender: state_tx,
                address: address.downgrade(),
            },
            from_context: state_rx,
            state: Default::default(),
        };

        (me, address)
    }

    /// Construct a new shutdown handle to be able to remotely shutdown the actor
    pub fn shutdown_handle(&self) -> ShutdownHandle {
        let sender = self.context.sender.clone();
        ShutdownHandle(sender)
    }
}

enum Race<A> {
    State(State),
    Envelope(Envelope<A>),
}

impl<A> Executor<A>
where
    A: Actor,
{
    /// Runs the executor, returns `Ok(())` if the actor invoked shutdown manually, and `Err(_)`
    /// if all addresses to the actor have been dropped
    ///
    /// This function should be likely be handed off to the spawn function of your async runtime
    /// of choice.
    pub async fn run(&mut self) -> Result<(), AddressError> {
        self.reset_state();
        self.actor.starting(&self.context).await;

        // TODO: In the future we will likely add more states, this is fine for now
        #[allow(clippy::while_let_loop)]
        let result = loop {
            match self.state {
                State::Continue => self.continuation().await,
                State::Shutdown => break Ok(()),
                State::SendersClosed => break Err(AddressError::Closed),
            }
        };

        self.actor.stopping(&self.context).await;

        result
    }

    /// Runs the executor, halting execution early if the provided future polls ready.
    ///
    /// Returns `Ok(true)` if the provided future resolved, and `Ok(false)` if the the actor was
    /// shut down
    ///
    /// This can be used in conjunction with [`Self::actor_mut`] to periodically alter the state of
    /// the actor.
    pub async fn run_against<F>(&mut self, fut: F) -> Result<bool, AddressError>
    where
        F: Future<Output = ()>,
    {
        self.reset_state();
        let fut1 = async { self.run().await.map(|_| false) };
        let fut2 = async {
            fut.await;
            Ok(true)
        };

        crate::futures::race_biased(fut1, fut2).await
    }

    /// Resets the actor's state
    fn reset_state(&mut self) {
        while self.from_context.try_recv().is_ok() {}
        self.state = State::Continue;
    }

    pub fn actor_ref(&self) -> &A {
        &self.actor
    }

    pub fn actor_mut(&mut self) -> &mut A {
        &mut self.actor
    }

    async fn continuation(&mut self) {
        let fut1 = async { self.from_context.recv().await.map(|val| Race::State(val)) };
        let fut2 = async { self.receiver.recv().await.map(|val| Race::Envelope(val)) };

        let result = crate::futures::race_biased(fut1, fut2).await;

        match result {
            Ok(Race::State(state)) => self.state = state,
            Ok(Race::Envelope(env)) => env.resolve(&mut self.actor, &self.context).await,
            Err(_) => {
                self.state = State::SendersClosed;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub struct Foo;

    impl Actor for Foo {}

    #[tokio::test]
    async fn dropped_address_exits() {
        let (mut actor, addr) = Executor::new(Foo);
        let handle = tokio::spawn(async move { actor.run().await });
        assert!(!handle.is_finished());
        drop(addr);
        assert!(handle.await.unwrap().is_err())
    }
}
