//! This is the threadpool crate for the VoxelRun project
//!
//! It utilizes a global threadpool which can be accessed with the global_init, global_exec and
//! global_promise functions.
//!
//! This crate also exposes a threadpool implementation.

pub mod promises;
pub mod threadpool;
mod latch;

pub use threadpool::{global_exec, global_init, global_promise};
