use std::any::{Any, TypeId};

use crate::Factory;

pub struct Overseer<T> {
    map: crate::ResourcePool,
    factories: Vec<Box<dyn Factory<Handle = T> + Send + Sync>>,
}

impl<T> Default for Overseer<T> {
    fn default() -> Self {
        Self {
            map: Default::default(),
            factories: Default::default(),
        }
    }
}

impl<T> Overseer<T> {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_resource<R>(&mut self, value: R) -> Option<R>
    where
        R: Any + Send + Sync,
    {
        let type_id = TypeId::of::<R>();
        let output = self.map.insert(value);

        for factory in &mut self.factories {
            factory.on_add(&self.map, &type_id);
        }

        output
    }

    pub fn insert_factory<F>(&mut self, factory: F)
    where
        F: Factory<Handle = T> + Send + Sync + 'static,
    {
        let factory = Box::new(factory);
        self.factories.push(factory);
    }

    pub fn remove_resource<R>(&mut self) -> Option<R>
    where
        R: Any + Send + Sync,
    {
        if !self.map.contains::<R>() {
            return None;
        }

        let type_id = TypeId::of::<R>();
        for factory in &mut self.factories {
            factory.on_remove(&self.map, &type_id)
        }

        self.map.remove::<R>()
    }
}
