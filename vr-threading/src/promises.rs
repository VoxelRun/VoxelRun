//! Module that describes the lazy execution type [Promise]

use std::sync::mpsc::{self, RecvError};

/// Represents a [Promise], promises are handles to lazily executed tasks
pub struct Promise<T> {
    indicator: mpsc::Receiver<T>,
}

impl<T> From<mpsc::Receiver<T>> for Promise<T> {
    fn from(value: mpsc::Receiver<T>) -> Self {
        Self { indicator: value }
    }
}

impl<T> Promise<T> {
    /// Get the value from a promise, may panic if the threadpool disconnected before the task was
    /// executed (effectively never)
    pub fn get(&self) -> T {
        self.indicator.recv().unwrap()
    }
    /// does the same thing as [Promise::get] but leaves the error behavious to the user
    pub fn try_get(&self) -> Result<T, RecvError> {
        self.indicator.recv()
    }
}
