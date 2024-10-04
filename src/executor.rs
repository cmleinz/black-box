use async_channel::Receiver;

use crate::{actors::Actor, message::Envelope, Address};

const DEFAULT_CAP: usize = 100;

pub struct Executor<A> {
    actor: A,
    receiver: Receiver<Envelope<A>>,
}

impl<A> Executor<A> {
    pub fn new(actor: A) -> (Self, Address<A>) {
        let (sender, receiver) = async_channel::bounded(DEFAULT_CAP);
        let me = Self { actor, receiver };
        let address = Address::new(sender);

        (me, address)
    }
}

impl<A> Executor<A>
where
    A: Actor,
{
    pub async fn run(mut self) {
        self.actor.starting().await;

        while let Ok(message) = self.receiver.recv().await {
            message.resolve(&mut self.actor).await;
        }

        self.actor.stopping().await;
    }
}
