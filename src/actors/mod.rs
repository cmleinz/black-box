use std::{any::Any, pin::Pin};

use crate::message::Message;

pub trait Actor {}

pub struct Address<A> {
    _phantom: std::marker::PhantomData<A>,
}

type FutType<A> = Box<
    dyn for<'a> FnOnce(
        &'a mut A,
        Box<dyn Any>,
    ) -> Pin<Box<dyn std::future::Future<Output = ()> + 'a>>,
>;

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

fn constraint<A, F>(fun: F) -> FutType<A>
where
    F: 'static
        + for<'a> FnOnce(
            &'a mut A,
            Box<dyn Any>,
        ) -> Pin<Box<dyn std::future::Future<Output = ()> + 'a>>,
{
    Box::new(fun)
}

impl<A: 'static + Actor> Address<A> {
    pub fn send<M>(&self, message: M)
    where
        A: Handler<M>,
        M: 'static + Message,
    {
        let content: Box<dyn Any> = Box::new(message);
        let data = constraint::<A, _>(|actor, msg| {
            let message: Box<M> = msg.downcast().unwrap();
            Box::pin(async move { actor.handle(*message).await })
        });

        let me = Envelope {
            mapping: Box::new(data),
            content,
        };
    }
}

pub trait Handler<M>
where
    Self: Actor,
    M: Message,
{
    fn handle(&mut self, msg: M) -> impl std::future::Future<Output = ()> + Send;
}
