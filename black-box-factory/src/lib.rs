pub mod overseer;
mod resource;

use std::any::TypeId;

pub use overseer::Overseer;
pub use resource::ResourcePool;

pub trait Handle {
    fn shutdown(&mut self, pool: &ResourcePool);
}

#[derive(Debug, Clone)]
pub struct ActorHandle(black_box::ShutdownHandle);

impl ActorHandle {
    pub fn new(value: black_box::ShutdownHandle) -> Self {
        Self::from(value)
    }
}

impl From<black_box::ShutdownHandle> for ActorHandle {
    fn from(value: black_box::ShutdownHandle) -> Self {
        Self(value)
    }
}

impl Handle for ActorHandle {
    fn shutdown(&mut self, _pool: &ResourcePool) {
        let _ = self.0.shutdown();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Action {
    /// The Overseer will keep any handle that exists in for it
    Noop,
    /// The Overseer shutdown the associated handle
    Shutdown,
    /// Shutdown the current actor if any, and rebuild the actor with build()
    Restart,
}

pub trait Factory {
    type Handle: Handle;

    fn build(&mut self, pool: &ResourcePool) -> Option<Self::Handle>;

    fn on_build(&mut self, _pool: &ResourcePool, _handle: &Self::Handle) -> Action {
        Action::Noop
    }

    fn on_insert(&mut self, _pool: &ResourcePool, _type_id: &TypeId) -> Action {
        Action::Noop
    }

    fn on_update(&mut self, _pool: &ResourcePool, _type_id: &TypeId) -> Action {
        Action::Noop
    }

    fn on_remove(&mut self, _pool: &ResourcePool, _type_id: &TypeId) -> Action {
        Action::Noop
    }
}
