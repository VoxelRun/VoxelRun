use std::sync::mpsc;

use crate::promises::Promise;

pub trait ThreadPool {
    fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static;

    fn promised<F, T>(&self, f: F) -> Promise<T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static;
}
