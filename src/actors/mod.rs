use std::{any::Any, future::Future, pin::Pin};

use crate::message::Message;

pub trait Actor {
    fn starting(&mut self) -> impl Future<Output = ()> + Send {
        std::future::ready(())
    }

    fn stopping(&mut self) -> impl Future<Output = ()> + Send {
        std::future::ready(())
    }
}

#[derive(Clone)]
pub struct Address<A> {
    sender: async_channel::Sender<Envelope<A>>,
}

type FutType<A> =
    Box<dyn for<'a> FnOnce(&'a mut A, Box<dyn Any>) -> Pin<Box<dyn Future<Output = ()> + 'a>>>;

pub struct Envelope<A> {
    content: Box<dyn Any>,
    mapping: FutType<A>,
}

impl<A> Envelope<A> {
    pub async fn resolve(self, actor: &mut A) {
        let fut = (self.mapping)(actor, self.content);
        fut.await;
    }
}

fn constrain<A, F>(fun: F) -> FutType<A>
where
    F: 'static + for<'a> FnOnce(&'a mut A, Box<dyn Any>) -> Pin<Box<dyn Future<Output = ()> + 'a>>,
{
    Box::new(fun)
}

impl<A: 'static + Actor> Address<A> {
    pub async fn send<M>(&self, message: M)
    where
        A: Handler<M>,
        M: 'static + Message,
    {
        let content: Box<dyn Any> = Box::new(message);
        let data = constrain::<A, _>(|actor, msg| {
            let message: Box<M> = msg.downcast().unwrap();
            Box::pin(async move { actor.handle(*message).await })
        });

        let env = Envelope {
            mapping: Box::new(data),
            content,
        };

        self.sender.send(env).await;
    }
}

pub trait Handler<M>
where
    Self: Actor,
    M: Message,
{
    fn handle(&mut self, msg: M) -> impl Future<Output = ()> + Send;
}
