use std::{num::NonZeroUsize, thread,sync::{atomic::AtomicUsize, Condvar, Mutex, RwLock}, cell::RefCell};

use crossbeam::{deque::{Injector, Stealer, Worker}, channel::{Sender, Receiver}};

pub struct Spawner {
    thread_no: NonZeroUsize,
}

thread_local! {
    pub static WORKSTEALING_QUEUE: RefCell<Option<Worker<Job>>> = RefCell::new(None);
}

pub struct Registry {
    pub stealers: RwLock<Vec<Stealer<Job>>>,
    // workload: Vec<AtomicUsize>,
    // conds: Vec<Condvar>,
    // with_work: Mutex<usize>,
}

impl Registry {
    pub fn register_worker(&self, stealer: Stealer<Job>) -> usize {
        let mut stealers = self.stealers.write().expect("Failed to acquire rwlock"); 
        stealers.push(stealer);
        stealers.len() - 1
    }
}

pub struct Job {
    pub function: Box<dyn FnOnce() + Send>,
}

impl Spawner {
    pub fn new(thread_no: NonZeroUsize) -> Self {
        Self { thread_no }
    }

    pub fn with_main<F, R>(self, main: F) -> R
    where
        F: FnOnce() -> R,
    {
        let registry = Registry { stealers: vec![].into() };
        todo!()
    }
}
