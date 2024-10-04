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
    data: FutType<A>,
}

fn fuckery<A, F>(fun: F) -> FutType<A>
where
    F: 'static
        + for<'a> FnOnce(
            &'a mut A,
            Box<dyn Any>,
        ) -> Pin<Box<dyn std::future::Future<Output = ()> + 'a>>,
{
    Box::new(fun)
}

impl<A: Actor> Address<A> {
    pub fn send<M>(&self, message: M)
    where
        A: Handler<M>,
        M: 'static + Message,
    {
        let data = fuckery::<A>(|actor, msg| {
            let message: Box<M> = msg.downcast().unwrap();
            Box::pin(async move { actor.handle(*message).await })
        });

        let me = Envelope {
            data: Box::new(data),
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
