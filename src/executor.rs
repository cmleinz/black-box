use std::any::{Any, TypeId};

use async_channel::Receiver;

use crate::{actors::Actor, message::Message};

pub struct Executor<A> {
    actor: A,
    receiver: Receiver<(TypeId, Box<dyn Any>)>,
}

impl<A> Executor<A>
where
    A: Actor,
{
    async fn run(&mut self) {
        while let Ok((id, message)) = self.receiver.recv().await {}
    }
}
