use crate::{Factory, ResourcePool};
use std::any::TypeId;

pub(crate) struct FactoryHolder<T> {
    factory: Box<dyn Factory<Handle = T> + Send + Sync>,
    autobuild: bool,
    built: bool,
}

pub(crate) struct FactorySet<T> {
    factories: Vec<FactoryHolder<T>>,
}

impl<T> Default for FactorySet<T> {
    fn default() -> Self {
        Self {
            factories: Vec::new(),
        }
    }
}

impl<T> FactorySet<T> {
    pub(crate) fn insert<F>(&mut self, factory: F, autobuild: bool)
    where
        F: Factory<Handle = T> + Send + Sync + 'static,
    {
        self.factories.push(FactoryHolder {
            factory: Box::new(factory),
            autobuild,
            built: false,
        });
    }

    pub(crate) fn on_add(&mut self, pool: &ResourcePool, type_id: &TypeId) -> Vec<T> {
        let mut new_handles = Vec::new();
        for holder in &mut self.factories {
            holder.factory.on_add(pool, type_id);
            if holder.autobuild && !holder.built {
                if let Some(handle) = holder.factory.build(pool) {
                    new_handles.push(handle);
                    holder.built = true;
                }
            }
        }
        new_handles
    }

    pub(crate) fn on_remove(&mut self, pool: &ResourcePool, type_id: &TypeId) {
        for holder in &mut self.factories {
            holder.factory.on_remove(pool, type_id);
        }
    }
}
