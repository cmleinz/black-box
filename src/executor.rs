use std::task::Poll;

use async_channel::Receiver;
use futures_util::{future::FusedFuture, Future};

use crate::{actors::Actor, message::Envelope, Address};

const DEFAULT_CAP: usize = 100;

struct FusedReceiver<T> {
    receiver: Receiver<T>,
}

impl<T> From<Receiver<T>> for FusedReceiver<T> {
    fn from(value: Receiver<T>) -> Self {
        FusedReceiver { receiver: value }
    }
}

impl<T> FusedReceiver<T> {
    fn recv(&self) -> FusedRecv<'_, T> {
        FusedRecv::new(&self.receiver)
    }
}

pub struct Executor<A> {
    actor: A,
    context: Context,
    state: State,
    from_context: FusedReceiver<State>,
    receiver: FusedReceiver<Envelope<A>>,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum State {
    #[default]
    Continue,
    Shutdown,
}

pin_project_lite::pin_project! {
    struct FusedRecv<'a, T> {
        #[pin]
        recv: async_channel::Recv<'a, T>,
        finished: bool,
    }
}

impl<'a, T> FusedRecv<'a, T> {
    fn new(recv: &'a Receiver<T>) -> Self {
        Self {
            recv: recv.recv(),
            finished: false,
        }
    }
}

impl<'a, T> Future for FusedRecv<'a, T> {
    type Output = Result<T, async_channel::RecvError>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.project();
        match this.recv.poll(cx) {
            std::task::Poll::Ready(Err(e)) => {
                *this.finished = true;
                Poll::Ready(Err(e))
            }
            other => other,
        }
    }
}

impl<'a, T> FusedFuture for FusedRecv<'a, T> {
    fn is_terminated(&self) -> bool {
        self.finished
    }
}

#[derive(Clone, Debug)]
pub struct Context {
    sender: async_channel::Sender<State>,
}

impl Context {
    /// Triggers the end of the executor
    pub fn shutdown(&mut self) {
        let _ = self.sender.force_send(State::Shutdown);
    }
}

impl<A> Executor<A> {
    pub fn new(actor: A) -> (Self, Address<A>) {
        let (sender, receiver) = async_channel::bounded(DEFAULT_CAP);
        let (state_tx, state_rx) = async_channel::unbounded();
        let me = Self {
            actor,
            receiver: receiver.into(),
            context: Context { sender: state_tx },
            from_context: state_rx.into(),
            state: Default::default(),
        };
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

        loop {
            match self.state {
                State::Continue => self.continuation().await,
                State::Shutdown => break,
            }
        }

        self.actor.stopping().await;
    }

    async fn continuation(&mut self) {
        futures_util::select! {
            state = self.from_context.recv() => {
                self.state = state.unwrap_or(State::Shutdown);
            }
            message = self.receiver.recv() => {
                let Ok(message) = message else {
                    self.state = State::Shutdown;
                    return
                };

                message.resolve(&mut self.actor, &self.context).await;
            }
        };
    }
}
