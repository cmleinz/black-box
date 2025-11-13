mod factory;
mod resource;

pub use factory::{InsertResource, Overseer, RemoveResource, UpdateResource};
pub use resource::ResourcePool;

pub trait Handle {
    fn shutdown(&mut self, pool: &mut ResourcePool);
}

pub trait Factory {
    type Handle;

    fn build(&mut self, pool: &ResourcePool) -> Option<Self::Handle>;

    fn on_build(&mut self, _pool: &ResourcePool, _handle: &Self::Handle) {}

    fn on_add(&mut self, _pool: &ResourcePool) {}

    fn on_update(&mut self, _pool: &ResourcePool) {}

    fn on_remove(&mut self, _pool: &ResourcePool) {}
}
