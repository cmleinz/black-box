use std::{future::Future, sync::atomic::AtomicU64};

use async_channel::{Sender, WeakSender};

use crate::{
    executor::Context,
    message::{Envelope, Message},
};

static ADDRESS_COUNTER: AtomicU64 = AtomicU64::new(0);

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
    id: u64,
    sender: Sender<Envelope<A>>,
}

impl<A> PartialEq for Address<A> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
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
            id: self.id,
        }
    }
}

impl<A> Address<A> {
    pub(crate) fn new(sender: Sender<Envelope<A>>) -> Self {
        let id = ADDRESS_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        Self { sender, id }
    }

    pub fn downgrade(&self) -> WeakAddress<A> {
        let sender = self.sender.downgrade();
        WeakAddress::new(self.id, sender)
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
    id: u64,
    sender: WeakSender<Envelope<A>>,
}

impl<A> Clone for WeakAddress<A> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            sender: self.sender.clone(),
        }
    }
}

impl<A> WeakAddress<A> {
    pub(crate) fn new(id: u64, sender: WeakSender<Envelope<A>>) -> Self {
        Self { id, sender }
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

#[cfg(test)]
mod test {
    use std::sync::Mutex;

    use crate::Executor;

    use super::*;

    struct Msg;
    struct Act;
    impl Actor for Act {}
    impl Handler<Msg> for Act {
        async fn handle(&mut self, _msg: Msg, _ctx: &Context<Self>) {}
    }

    #[test]
    fn partial_eq_on_clone() {
        let (_executor, address) = Executor::new(Act);
        let same_address = address.clone();
        assert!(address.eq(&same_address));
    }

    #[test]
    fn partial_eq_on_different_addrs() {
        let (_executor_1, address_1) = Executor::new(Act);
        let (_executor_2, address_2) = Executor::new(Act);
        assert!(address_1.ne(&address_2));
    }

    #[test]
    fn partial_eq_on_a_thousand_different_addrs() {
        let mut addrs: Vec<Address<Act>> = Vec::new();
        for _ in 0..1_000 {
            let (_executor_1, address) = Executor::new(Act);
            for addr in addrs.iter() {
                assert!(addr.ne(&address));
            }
            addrs.push(address);
        }
    }

    #[test]
    fn partial_eq_on_a_thousand_different_threads() {
        const NUM_THREAD: usize = 1_000;
        let addrs = Mutex::new(Vec::<Address<Act>>::new());
        std::thread::scope(|s| {
            for _ in 0..NUM_THREAD {
                s.spawn(|| {
                    let (_executor_1, address) = Executor::new(Act);
                    addrs.lock().unwrap().push(address);
                });
            }
        });
        let addrs = std::mem::take(&mut *addrs.lock().unwrap());
        assert_eq!(addrs.len(), NUM_THREAD);
        for i in 0..NUM_THREAD {
            for j in (i + 1)..NUM_THREAD {
                assert!(addrs[i].ne(&addrs[j]))
            }
        }
    }
}
