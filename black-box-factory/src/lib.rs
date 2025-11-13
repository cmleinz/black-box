mod factory;
mod factory_set;
mod resource;

use std::any::{Any, TypeId};

pub use factory::{InsertResource, Overseer, RemoveResource, UpdateResource};
pub use resource::ResourcePool;

pub trait Factory {
    type Handle;

    fn build(&mut self, pool: &ResourcePool) -> Option<Self::Handle>;

    fn on_add(&mut self, _pool: &ResourcePool, _type_id: &TypeId) {}

    fn on_update(&mut self, _pool: &ResourcePool, _type_id: &TypeId, _value: &dyn Any) {}

    fn on_remove(&mut self, _pool: &ResourcePool, _type_id: &TypeId) {}
}
