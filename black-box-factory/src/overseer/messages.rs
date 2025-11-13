use std::{any::Any, marker::PhantomData};

use black_box::Handler;

use crate::Handle;

use super::Overseer;

/// Adds a new resource to the Overseer
pub struct InsertResource<U>(U);

impl<U> InsertResource<U> {
    pub fn new(value: U) -> Self {
        Self(value)
    }
}

impl<T, U> Handler<InsertResource<U>> for Overseer<T>
where
    U: Any + Send + Sync,
    T: Handle + Send,
{
    async fn handle(&mut self, msg: InsertResource<U>, _ctx: &black_box::Context<Self>) {
        self.insert_resource(msg.0);
    }
}

/// Removes a resource
pub struct RemoveResource<U>(PhantomData<U>);

impl<U> Default for RemoveResource<U> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<U> RemoveResource<U> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<T, U> Handler<RemoveResource<U>> for Overseer<T>
where
    U: Any + Send + Sync,
    T: Handle + Send,
{
    async fn handle(&mut self, _msg: RemoveResource<U>, _ctx: &black_box::Context<Self>) {
        self.remove_resource::<U>();
    }
}

/// Replaces an existing resource in the Overseer
pub struct UpdateResource<U>(U);

impl<U> UpdateResource<U> {
    pub fn new(value: U) -> Self {
        Self(value)
    }
}

impl<T, U> Handler<UpdateResource<U>> for Overseer<T>
where
    U: Any + Send + Sync,
    T: Handle + Send,
{
    async fn handle(&mut self, msg: UpdateResource<U>, _ctx: &black_box::Context<Self>) {
        self.update_resource(msg.0);
    }
}
