use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Key {
    id: TypeId,
}

impl Key {
    fn of<T: Any>() -> Self {
        Self {
            id: TypeId::of::<T>(),
        }
    }
}

struct Resource {
    value: Box<dyn Any + Send + Sync>,
}

#[derive(Default)]
pub struct ResourcePool {
    map: HashMap<Key, Resource>,
}

impl ResourcePool {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn contains<T: Any>(&self) -> bool {
        let key = Key::of::<T>();
        self.map.contains_key(&key)
    }

    #[inline]
    pub fn contains_id(&self, id: TypeId) -> bool {
        let key = Key { id };
        self.map.contains_key(&key)
    }

    #[inline]
    pub fn insert<T>(&mut self, value: T) -> Option<T>
    where
        T: Any + Send + Sync,
    {
        let key = Key::of::<T>();
        let resource = Resource {
            value: Box::new(value),
        };

        let was = self.remove_with_key(&key);
        self.map.insert(key, resource);
        was
    }

    #[inline]
    pub fn get<T: Any>(&self) -> Option<&T> {
        let key = Key::of::<T>();
        self.map.get(&key).and_then(|t| t.value.downcast_ref())
    }

    #[inline]
    pub fn get_clone<T: Any + Clone>(&self) -> Option<T> {
        let key = Key::of::<T>();
        self.map
            .get(&key)
            .and_then(|t| t.value.downcast_ref())
            .cloned()
    }

    #[inline]
    pub fn get_mut<T: Any>(&mut self) -> Option<&mut T> {
        let key = Key::of::<T>();
        self.map.get_mut(&key).and_then(|t| t.value.downcast_mut())
    }

    #[inline]
    pub fn remove<T: Any>(&mut self) -> Option<T> {
        let key = Key::of::<T>();
        self.remove_with_key(&key)
    }

    #[inline]
    fn remove_with_key<T: Any>(&mut self, key: &Key) -> Option<T> {
        self.map
            .remove(key)
            .and_then(|t| t.value.downcast().ok().map(|boxed| *boxed))
    }
}
