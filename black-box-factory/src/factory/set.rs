use crate::{Factory, ResourcePool};

pub(super) struct FactoryHolder<T> {
    factory: Box<dyn Factory<Handle = T> + Send + Sync>,
    autobuild: bool,
    built: bool,
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
            built: false,
        });
    }

    pub(super) fn on_update(&mut self, pool: &ResourcePool) {
        for holder in &mut self.factories {
            holder.factory.on_update(pool);
        }
    }

    pub(super) fn on_add(&mut self, pool: &ResourcePool) -> Vec<T> {
        let mut new_handles = Vec::new();
        for holder in &mut self.factories {
            holder.factory.on_add(pool);
            if holder.autobuild && !holder.built {
                if let Some(handle) = holder.factory.build(pool) {
                    holder.factory.on_build(pool, &handle);
                    new_handles.push(handle);
                    holder.built = true;
                }
            }
        }
        new_handles
    }

    pub(super) fn on_remove(&mut self, pool: &ResourcePool) {
        for holder in &mut self.factories {
            holder.factory.on_remove(pool);
        }
    }
}
