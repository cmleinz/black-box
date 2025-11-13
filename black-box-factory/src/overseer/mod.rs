use black_box::Actor;
use std::any::{Any, TypeId};

use crate::{Factory, Handle};

pub mod messages;
mod set;

use set::FactorySet;

pub struct Overseer<T> {
    map: crate::ResourcePool,
    factory_set: FactorySet<T>,
}

impl<T> Actor for Overseer<T> {}

impl<T> Default for Overseer<T> {
    fn default() -> Self {
        Self {
            map: Default::default(),
            factory_set: Default::default(),
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
}

impl<T> Overseer<T>
where
    T: Handle,
{
    pub fn insert_resource<R>(&mut self, value: R) -> Option<R>
    where
        R: Any + Send + Sync,
    {
        let output = self.map.insert(value);
        let id = TypeId::of::<R>();

        self.factory_set.on_add(&self.map, &id);

        output
    }

    pub fn update_resource<R>(&mut self, value: R) -> Option<R>
    where
        R: Any + Send + Sync,
    {
        let was = self.map.get_mut::<R>()?;
        let was = std::mem::replace(was, value);
        let id = TypeId::of::<R>();

        self.factory_set.on_update(&self.map, &id);

        Some(was)
    }

    pub fn remove_resource<R>(&mut self) -> Option<R>
    where
        R: Any + Send + Sync,
    {
        if !self.map.contains::<R>() {
            return None;
        }
        let id = TypeId::of::<R>();

        self.factory_set.on_remove(&self.map, &id);

        self.map.remove::<R>()
    }
}
