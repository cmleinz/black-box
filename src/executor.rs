use async_channel::Receiver;

use crate::actors::{Actor, Envelope};

pub struct Executor<A> {
    actor: A,
    receiver: Receiver<Envelope<A>>,
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
