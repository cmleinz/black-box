use std::{any::Any, future::Future, pin::Pin};

use crate::{executor::Context, Handler};

pub trait Message {
    type Response;
}

type FutType<A> = Box<
    dyn for<'a> FnOnce(
            &'a mut A,
            Box<dyn Any>,
            &'a Context,
        ) -> Pin<Box<dyn Future<Output = ()> + 'a>>
        + Send,
>;

pub(crate) struct Envelope<A> {
    content: Box<dyn Any>,
    mapping: FutType<A>,
}

impl<A> Envelope<A> {
    pub(crate) fn pack<M>(message: M) -> Self
    where
        M: 'static + Message,
        A: 'static + Handler<M>,
    {
        let content: Box<dyn Any> = Box::new(message);
        let mapping = Self::constrain(|actor, msg, ctx| {
            let message: Box<M> = msg.downcast().unwrap();
            Box::pin(async move { actor.handle(*message, ctx).await })
        });
        let mapping = Box::new(mapping);

        Self { content, mapping }
    }

    pub(crate) fn unpack<M: 'static>(self) -> M {
        let value = self.content.downcast().unwrap();
        *value
    }

    fn constrain<F>(fun: F) -> FutType<A>
    where
        F: 'static
            + for<'a> FnOnce(
                &'a mut A,
                Box<dyn Any>,
                &'a Context,
            ) -> Pin<Box<dyn Future<Output = ()> + 'a>>
            + Send,
    {
        Box::new(fun)
    }

    pub(crate) async fn resolve(self, actor: &mut A, ctx: &Context) {
        let fut = (self.mapping)(actor, self.content, ctx);
        fut.await;
    }
}
