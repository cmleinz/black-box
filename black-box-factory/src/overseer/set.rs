use std::any::TypeId;

use crate::{Action, Factory, Handle, ResourcePool};

pub(super) struct FactoryHolder<T> {
    factory: Box<dyn Factory<Handle = T> + Send + Sync>,
    autobuild: bool,
    handle: Option<T>,
}

impl<T> FactoryHolder<T>
where
    T: Handle,
{
    fn handle_action(&mut self, action: Action, pool: &ResourcePool) {
        match action {
            Action::Noop => (),
            Action::Shutdown => {
                if let Some(mut handle) = self.handle.take() {
                    handle.shutdown(pool);
                }
            }
            Action::Restart => {
                if let Some(mut handle) = self.handle.take() {
                    handle.shutdown(pool);
                }
                self.handle = self.factory.build(pool);
            }
        }
    }
}

pub(super) struct FactorySet<T> {
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
    pub(super) fn insert<F>(&mut self, factory: F, autobuild: bool)
    where
        F: Factory<Handle = T> + Send + Sync + 'static,
    {
        self.factories.push(FactoryHolder {
            factory: Box::new(factory),
            autobuild,
            handle: None,
        });
    }
}
impl<T> FactorySet<T>
where
    T: Handle,
{
    pub(super) fn on_update(&mut self, pool: &ResourcePool, id: &TypeId) {
        for holder in &mut self.factories {
            let action = holder.factory.on_update(pool, id);
            holder.handle_action(action, pool);
        }
    }

    pub(super) fn on_add(&mut self, pool: &ResourcePool, id: &TypeId) {
        for holder in &mut self.factories {
            holder.factory.on_insert(pool, id);
            if holder.autobuild && holder.handle.is_none() {
                if let Some(handle) = holder.factory.build(pool) {
                    let action = holder.factory.on_build(pool, &handle);
                    holder.handle = Some(handle);
                    holder.handle_action(action, pool);
                }
            }
        }
    }

    pub(super) fn on_remove(&mut self, pool: &ResourcePool, id: &TypeId) {
        for holder in &mut self.factories {
            let action = holder.factory.on_remove(pool, id);
            holder.handle_action(action, pool);
        }
    }
}
