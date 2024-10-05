use std::{any::Any, future::Future, pin::Pin};

use crate::{executor::Context, Handler};

pub trait Message: 'static + Send {}

impl<T: 'static + Send> Message for T {}

type FutType<A> = Box<
    dyn for<'a> FnOnce(
            &'a mut A,
            Box<dyn Any + Send>,
            &'a Context,
        ) -> Pin<Box<dyn Future<Output = ()> + 'a + Send>>
        + Send,
>;

pub(crate) struct Envelope<A> {
    content: Box<dyn Any + Send>,
    mapping: FutType<A>,
}

impl<A> Envelope<A> {
    pub(crate) fn pack<M>(message: M) -> Self
    where
        M: Message,
        A: 'static + Handler<M> + Send,
    {
        let content: Box<dyn Any + Send> = Box::new(message);
        let mapping = Self::constrain(|actor, msg, ctx| {
            let message = Self::unpack(msg);
            Box::pin(async move { actor.handle(message, ctx).await })
        });
        let mapping = Box::new(mapping);

        Self { content, mapping }
    }

    pub(crate) fn unpack<M: 'static>(val: Box<dyn Any>) -> M {
        let value = val.downcast().unwrap();
        *value
    }

    fn constrain<F>(fun: F) -> FutType<A>
    where
        F: 'static
            + for<'a> FnOnce(
                &'a mut A,
                Box<dyn Any + Send>,
                &'a Context,
            ) -> Pin<Box<dyn Future<Output = ()> + 'a + Send>>
            + Send,
    {
        Box::new(fun)
    }

    pub(crate) async fn resolve(self, actor: &mut A, ctx: &Context) {
        let fut = (self.mapping)(actor, self.content, ctx);
        fut.await;
    }
}
