#![doc = include_str!("../README.md")]

mod actors;
pub mod error;
mod executor;
mod futures;
pub(crate) mod message;

pub use self::{
    actors::{Actor, Address, Handler, WeakAddress},
    executor::{Context, Executor},
};
