use std::future::Future;

use async_channel::{Sender, WeakSender};

use crate::{
    executor::Context,
    message::{Envelope, Message},
};

/// Abstraction for message handleing
///
/// Actors are spawned in an [`Executor`](crate::Executor), and run in the executor's event loop.
/// When new messages are received by the executor, the appropriate handler [`Handler`] is invoked,
/// allowing the actor to take any necessary action, including mutating it's internal state.
pub trait Actor: Sized {
    fn starting(&mut self, _ctx: &Context<Self>) -> impl Future<Output = ()> + Send {
        std::future::ready(())
    }

    fn stopping(&mut self, _ctx: &Context<Self>) -> impl Future<Output = ()> + Send {
        std::future::ready(())
    }
}

/// The implementation for how an actor handles a particular message
///
/// An [`Actor`], can implement the Handler trait any number of time, with a unique message type for
/// each implementation.
pub trait Handler<M>
where
    Self: Actor,
    M: Message,
{
    /// Asynchronously act on the message, with mutable access to self
    fn handle(&mut self, msg: M, ctx: &Context<Self>) -> impl Future<Output = ()> + Send;
}

/// A cloneable address which can be used to send messages to the associated [`Actor`]
///
/// This is a cheaply cloneable type and can be used to send an actor address to other actors, other
/// runtimes, etc.
#[derive(Debug)]
pub struct Address<A> {
    sender: Sender<Envelope<A>>,
}

// SAFETY: The address is a queue abstraction for *messages* sent to the actor. Even if the actor
// itself is not Send/Sync the address should be. The Message trait itself already requires that
// the implementer be Send
unsafe impl<A> std::marker::Send for Address<A> {}
// SAFETY: As above but for Sync
unsafe impl<A> std::marker::Sync for Address<A> {}
impl<A> std::marker::Unpin for Address<A> {}

impl<A> Clone for Address<A> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

impl<A> Address<A> {
    pub(crate) fn new(sender: Sender<Envelope<A>>) -> Self {
        Self { sender }
    }

    pub fn downgrade(&self) -> WeakAddress<A> {
        let sender = self.sender.downgrade();
        WeakAddress::new(sender)
    }
}

impl<A> Address<A>
where
    A: 'static + Actor + Send,
{
    /// Send the given message to the actor's receiver.
    ///
    /// If the receiver is currently full, it will await capacity to enqueue the message
    pub async fn send<M>(&self, message: M)
    where
        A: Handler<M>,
        M: Message,
    {
        let env = Envelope::pack(message);

        // TODO: Decide what to do here
        let _ = self.sender.send(env).await;
    }

    pub fn try_send<M>(&self, message: M)
    where
        A: Handler<M>,
        M: Message,
    {
        let env = Envelope::pack(message);

        // TODO: Decide what to do here
        let _ = self.sender.try_send(env);
    }
}

/// A cloneable address which can be used to send messages to the associated [`Actor`]
///
/// This is a cheaply cloneable type and can be used to send an actor address to other actors, other
/// runtimes, etc.
#[derive(Debug)]
pub struct WeakAddress<A> {
    sender: WeakSender<Envelope<A>>,
}

impl<A> Clone for WeakAddress<A> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

impl<A> WeakAddress<A> {
    pub(crate) fn new(sender: WeakSender<Envelope<A>>) -> Self {
        Self { sender }
    }

    pub fn upgrade(&self) -> Option<Address<A>> {
        let sender = self.sender.upgrade()?;
        Some(Address::new(sender))
    }
}

// SAFETY: The address is a queue abstraction for *messages* sent to the actor. Even if the actor
// itself is not Send/Sync the address should be. The Message trait itself already requires that
// the implementer be Send
unsafe impl<A> std::marker::Send for WeakAddress<A> {}
// SAFETY: As above but for Sync
unsafe impl<A> std::marker::Sync for WeakAddress<A> {}
impl<A> std::marker::Unpin for WeakAddress<A> {}
