mod factory;
mod factory_set;
mod resource;

use std::any::TypeId;

pub use factory::Overseer;
pub use resource::ResourcePool;

pub trait Factory {
    type Handle;

    fn build(&mut self, pool: &ResourcePool) -> Option<Self::Handle>;

    fn on_add(&mut self, _pool: &ResourcePool, _type_id: &TypeId) {}

    fn on_remove(&mut self, _pool: &ResourcePool, _type_id: &TypeId) {}
}
