use std::{any::Any, marker::PhantomData};

use black_box::Handler;

use crate::Handle;

use super::Overseer;

/// Adds a new resource to the Overseer
///
/// If the resource already exists, do nothing
pub struct InsertResource<R>(R);

impl<R> InsertResource<R> {
    pub fn new(value: R) -> Self {
        Self(value)
    }
}

impl<T, R> Handler<InsertResource<R>> for Overseer<T>
where
    R: Any + Send + Sync,
    T: Handle + Send,
{
    async fn handle(&mut self, msg: InsertResource<R>, _ctx: &black_box::Context<Self>) {
        if !self.contains_resource::<R>() {
            self.insert_resource(msg.0);
        }
    }
}

/// Removes a resource
pub struct RemoveResource<R>(PhantomData<R>);

impl<R> Default for RemoveResource<R> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<R> RemoveResource<R> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<T, R> Handler<RemoveResource<R>> for Overseer<T>
where
    R: Any + Send + Sync,
    T: Handle + Send,
{
    async fn handle(&mut self, _msg: RemoveResource<R>, _ctx: &black_box::Context<Self>) {
        self.remove_resource::<R>();
    }
}

/// If the resource exists, replaces it with the provided one, calling on_update for all factories
///
/// Otherwise, do nothing
pub struct UpdateResource<R>(R);

impl<R> UpdateResource<R> {
    pub fn new(value: R) -> Self {
        Self(value)
    }
}

impl<T, R> Handler<UpdateResource<R>> for Overseer<T>
where
    R: Any + Send + Sync,
    T: Handle + Send,
{
    async fn handle(&mut self, msg: UpdateResource<R>, _ctx: &black_box::Context<Self>) {
        if self.contains_resource::<R>() {
            self.update_resource(msg.0);
        }
    }
}

/// If the resource exists, replaces it with the provided one, calling on_update for all factories
///
/// Otherwise, if the resource does not exist, it inserts it, calling on_insert for all factories
pub struct UpdateOrInsertResource<R>(R);

impl<R> UpdateOrInsertResource<R> {
    pub fn new(value: R) -> Self {
        Self(value)
    }
}

impl<T, R> Handler<UpdateOrInsertResource<R>> for Overseer<T>
where
    R: Any + Send + Sync,
    T: Handle + Send,
{
    async fn handle(&mut self, msg: UpdateOrInsertResource<R>, _ctx: &black_box::Context<Self>) {
        if self.contains_resource::<R>() {
            self.update_resource(msg.0);
        } else {
            self.insert_resource(msg.0);
        }
    }
}
