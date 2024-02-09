use std::{
    sync::Arc,
    thread::{self, JoinHandle},
};

use crossbeam::deque::Worker;

use crate::{Job, Registry, WORKSTEALING_QUEUE};

pub struct WorkerThread {
    shared: Arc<Registry>,
    // queue: Worker<Job>,
    index: usize,
}

impl WorkerThread {
    pub fn new(registry: Arc<Registry>) -> JoinHandle<()> {
        let queue = Worker::<Job>::new_fifo();
        let idx = registry.register_worker(queue.stealer());
        thread::spawn(move || {
            let worker_thread = Self {
                shared: registry,
                index: idx,
            };
            WORKSTEALING_QUEUE.with(|f| {
                *f.borrow_mut() = Some(queue);
            });
            WORKSTEALING_QUEUE.with(|f| {
                let local = f.borrow_mut();
                let local: &Worker<Job> = local.as_ref().unwrap();
                let job = {
                    let mut res = None;
                    while res.is_none() {
                        res = local.pop().or_else(|| {
                            let stealers =
                                worker_thread.shared.stealers.read().expect(
                                    "Failed to lock read write lock for acquiring stealers",
                                );
                            stealers
                                .iter()
                                .map(|s| s.steal())
                                .find(|s| s.is_success())
                                .and_then(|s| s.success())
                        });
                    }
                    res.unwrap()
                };
                (job.function)()
            });
        })
    }
}
