use std::future::Future;

use async_channel::Sender;

use crate::{
    executor::Context,
    message::{Envelope, Message},
};

pub trait Actor {
    fn starting(&mut self) -> impl Future<Output = ()> + Send {
        std::future::ready(())
    }

    fn stopping(&mut self) -> impl Future<Output = ()> + Send {
        std::future::ready(())
    }
}

pub trait Handler<M>
where
    Self: Actor,
    M: Message,
{
    fn handle(&mut self, msg: M, ctx: &Context) -> impl Future<Output = ()> + Send;
}

#[derive(Clone)]
pub struct Address<A> {
    sender: Sender<Envelope<A>>,
}

impl<A> Address<A> {
    pub(crate) fn new(sender: Sender<Envelope<A>>) -> Self {
        Self { sender }
    }
}

impl<A: 'static + Actor> Address<A> {
    pub async fn send<M>(&self, message: M)
    where
        A: Handler<M>,
        M: 'static + Message + Send,
    {
        let env = Envelope::pack(message);

        // TODO: Decide what to do here
        let _ = self.sender.send(env).await;
    }
}
