//! This is the threadpool crate for the VoxelRun project
//!
//! This crate exposes a workstealing threadpool implementation.
mod implementation;
mod poolworker;
pub use implementation::*;

pub trait ThreadPool {
    /// Starts two continous execution units and waits until both are finished returning the return
    /// values
    fn join<F1, F2, T1, T2>(task1: F1, task2: F2) -> (T1, T2)
    where
        F1: Send + FnOnce() -> T1,
        T1: Send,
        F2: Send + FnOnce() -> T2,
        T2: Send;

    /// Same as [ThreadPool::join] but doesn't care for return values
    fn join_ignore<F1, F2>(task1: F1, task2: F2)
    where
        F1: Send + FnOnce(),
        F2: Send + FnOnce();

    /// Joins over N threads, similar to [ThreadPool::join], although technical limitations don't
    /// allow for returning the return values that don't match
    fn join_all(tasks: Vec<Box<dyn FnOnce()>>);

    /// Same as [ThreadPool::join_all] but with the restriction that the function must have the
    /// same return value, but allows for the function to return the values in a vec
    fn join_all_same_return<T>(tasks: Vec<Box<dyn FnOnce() -> T>>) -> Vec<T>
    where
        T: Send;

    /// Same as [ThreadPool::join_all_same_return] but not using vtable but requiring same
    /// signature
    fn join_all_same_signature<F, T>(tasks: Vec<F>) -> Vec<T>
    where
        F: Send + FnOnce() -> T,
        T: Send;

    /// Spawns a task in the threadpool
    fn task<F>(task: F)
    where
        F: FnOnce() + Send;

    /// Spawns new thread into the threadpool with no queue responsible for running a new mainloop
    fn main_task<F>(task: F)
    where
        F: FnOnce() + Send;
}
