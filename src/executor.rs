use async_channel::Receiver;

use crate::{Actor, Address, WeakAddress, message::Envelope};

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
#[derive(Debug, Clone)]
pub struct Context<A> {
    sender: async_channel::Sender<State>,
    address: WeakAddress<A>,
}

impl<A> Context<A> {
    /// Triggers the end of the executor.
    ///
    /// Once triggered, no new messages will be processed and the actor will exit after resolving
    /// [`Actor::stopping`]
    pub fn shutdown(&self) {
        let _ = self.sender.force_send(State::Shutdown);
    }

    /// Retrieve the address for the executor's actor
    ///
    /// This is useful when an actor wants to emit messages to itself.
    pub fn address(&self) -> &WeakAddress<A> {
        &self.address
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
/// ```no_run
/// # use black_box::*;
/// # struct MyActor;
/// # impl Actor for MyActor {}
/// let my_actor = MyActor;
/// let (executor, addr) = Executor::new(my_actor);
///
/// tokio::spawn(executor.run());
/// ```
#[derive(Debug)]
pub struct Executor<A> {
    actor: A,
    context: Context<A>,
    state: State,
    from_context: Receiver<State>,
    receiver: Receiver<Envelope<A>>,
}

impl<A> Executor<A> {
    pub fn new(actor: A) -> (Self, Address<A>) {
        let (sender, receiver) = async_channel::bounded(DEFAULT_CAP);
        let address = Address::new(sender);
        let (state_tx, state_rx) = async_channel::unbounded();
        let me = Self {
            actor,
            receiver,
            context: Context {
                sender: state_tx,
                address: address.downgrade(),
            },
            from_context: state_rx,
            state: Default::default(),
        };

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
        self.actor.starting(&self.context).await;

        // TODO: In the future we will likely add more states, this is fine for now
        #[allow(clippy::while_let_loop)]
        loop {
            match self.state {
                State::Continue => self.continuation().await,
                State::Shutdown => break,
            }
        }

        self.actor.stopping(&self.context).await;
    }

    async fn continuation(&mut self) {
        let fut1 = async { self.from_context.recv().await.map(|val| Race::State(val)) };
        let fut2 = async { self.receiver.recv().await.map(|val| Race::Envelope(val)) };

        let result = crate::futures::race_biased(fut1, fut2).await;

        match result {
            Ok(Race::State(state)) => self.state = state,
            Ok(Race::Envelope(env)) => env.resolve(&mut self.actor, &self.context).await,
            Err(_) => {
                self.state = State::Shutdown;
            }
        }
    }
}
