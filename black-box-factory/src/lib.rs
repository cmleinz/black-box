mod factory;
mod resource;

pub use factory::{InsertResource, Overseer, RemoveResource, UpdateResource};
pub use resource::ResourcePool;

pub trait Handle {
    fn shutdown(&mut self, pool: &ResourcePool);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Action {
    /// The Overseer will keep any handle that exists in for it
    Noop,
    /// The Overseer shutdown the associated handle
    Shutdown,
}

pub trait Factory {
    type Handle: Handle;

    fn build(&mut self, pool: &ResourcePool) -> Option<Self::Handle>;

    fn on_build(&mut self, _pool: &ResourcePool, _handle: &Self::Handle) -> Action {
        Action::Noop
    }

    fn on_add(&mut self, _pool: &ResourcePool) -> Action {
        Action::Noop
    }

    fn on_update(&mut self, _pool: &ResourcePool) -> Action {
        Action::Noop
    }

    fn on_remove(&mut self, _pool: &ResourcePool) -> Action {
        Action::Noop
    }
}
