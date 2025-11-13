use std::any::{Any, TypeId};

use crate::factory_set::FactorySet;
use crate::Factory;

pub struct Overseer<T> {
    map: crate::ResourcePool,
    factory_set: FactorySet<T>,
    handles: Vec<T>,
}

impl<T> Default for Overseer<T> {
    fn default() -> Self {
        Self {
            map: Default::default(),
            factory_set: Default::default(),
            handles: Default::default(),
        }
    }
}

impl<T> Overseer<T> {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn contains_resource<R>(&self) -> bool
    where
        R: Any,
    {
        self.map.contains::<R>()
    }

    pub fn update_resource<R>(&mut self, value: R) -> Option<R>
    where
        R: Any + Send + Sync,
    {
        let was = self.map.insert(value);
        let now: &dyn Any = self.map.get::<R>().unwrap();
        let type_id = TypeId::of::<R>();

        self.factory_set.on_update(&self.map, &type_id, now);

        was
    }

    pub fn insert_resource<R>(&mut self, value: R) -> Option<R>
    where
        R: Any + Send + Sync,
    {
        let type_id = TypeId::of::<R>();
        let output = self.map.insert(value);

        let mut new_handles = self.factory_set.on_add(&self.map, &type_id);
        self.handles.append(&mut new_handles);

        output
    }

    pub fn insert_factory_manual<F>(&mut self, factory: F)
    where
        F: Factory<Handle = T> + Send + Sync + 'static,
    {
        self.factory_set.insert(factory, false);
    }

    pub fn insert_factory_autobuild<F>(&mut self, factory: F)
    where
        F: Factory<Handle = T> + Send + Sync + 'static,
    {
        self.factory_set.insert(factory, true);
    }

    pub fn remove_resource<R>(&mut self) -> Option<R>
    where
        R: Any + Send + Sync,
    {
        if !self.map.contains::<R>() {
            return None;
        }

        let type_id = TypeId::of::<R>();
        self.factory_set.on_remove(&self.map, &type_id);

        self.map.remove::<R>()
    }

    pub fn handles(&self) -> &[T] {
        &self.handles
    }
}
