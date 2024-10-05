#![doc = include_str!("../README.md")]

mod actors;
mod executor;
pub(crate) mod message;

pub use self::{
    actors::{Actor, Address, Handler},
    executor::{Context, Executor},
    message::Message,
};
