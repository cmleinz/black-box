use async_channel::Receiver;

use crate::{message::Envelope, Actor, Address};

const DEFAULT_CAP: usize = 100;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum State {
    #[default]
    Continue,
    Shutdown,
}

/// A cloneable context for the actor.
///
/// Currently this fuctions as a means by which to alter the state of the [`Executor`], it is
/// cloneable and can thus be sent to other threads, runtimes or even other actors to trigger a
/// shutdown.
#[derive(Clone, Debug)]
pub struct Context {
    sender: async_channel::Sender<State>,
}

impl Context {
    /// Triggers the end of the executor.
    ///
    /// Once triggered, no new messages will be processed and the actor will exit after resolving
    /// [`Actor::stopping`]
    pub fn shutdown(&self) {
        let _ = self.sender.force_send(State::Shutdown);
    }
}

/// The event loop for an actor
///
/// Handles the receipt of messages, and state management of the actor. The primary method exposed
/// by the executor is [`Executor::run`], which is used to execute the event loop.
///
/// # Example
///
/// A common pattern is to spawn the executor onto an async runtime like tokio.
///
/// ```ignore
/// let my_actor = MyActor;
/// let (executor, addr) = Executor::new(my_actor);
///
/// tokio::spawn(executor.run());
/// ```
pub struct Executor<A> {
    actor: A,
    context: Context,
    state: State,
    from_context: Receiver<State>,
    receiver: Receiver<Envelope<A>>,
}

impl<A> Executor<A> {
    pub fn new(actor: A) -> (Self, Address<A>) {
        let (sender, receiver) = async_channel::bounded(DEFAULT_CAP);
        let (state_tx, state_rx) = async_channel::unbounded();
        let me = Self {
            actor,
            receiver,
            context: Context { sender: state_tx },
            from_context: state_rx,
            state: Default::default(),
        };
        let address = Address::new(sender);

        (me, address)
    }
}

enum Race<A> {
    State(State),
    Envelope(Envelope<A>),
}

impl<A> Executor<A>
where
    A: Actor,
{
    pub async fn run(mut self) {
        self.actor.starting().await;

        // TODO: In the future we will likely add more states, this is fine for now
        #[allow(clippy::while_let_loop)]
        loop {
            match self.state {
                State::Continue => self.continuation().await,
                State::Shutdown => break,
            }
        }

        self.actor.stopping().await;
    }

    async fn continuation(&mut self) {
        let fut1 = async { self.from_context.recv().await.map(|val| Race::State(val)) };
        let fut2 = async { self.receiver.recv().await.map(|val| Race::Envelope(val)) };

        let result = futures_lite::future::race(fut1, fut2).await;

        match result {
            Ok(Race::State(state)) => self.state = state,
            Ok(Race::Envelope(env)) => env.resolve(&mut self.actor, &self.context).await,
            Err(_) => {
                self.state = State::Shutdown;
            }
        }
    }
}
