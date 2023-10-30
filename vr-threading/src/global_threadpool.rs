use std::sync::mpsc;

pub trait ThreadPool {
    fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static;

    fn execute_eval<F, T>(&self, f: F) -> mpsc::Receiver<T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static;
}
