//! Implementation of the threadpool

use std::{
    num::NonZeroUsize,
    sync::{mpsc, Arc, Mutex, Once},
    thread::{self, available_parallelism},
};

use vr_logger::{debug, error, info};

use crate::promises::Promise;

/// trait every threadpool needs to implement
pub trait ThreadPool {
    /// execute a function in the threadpool without garantee of it being finished and ignoring the
    /// return value
    fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static;

    /// get a handle to the result of the function in the form of a [Promise]
    fn promised<F, T>(&self, f: F) -> Promise<T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static;
}
type Job = Box<dyn FnOnce() + Send + 'static>;

const NO_THREADPOOL_ERROR: &str = "Tried to execute a task while threadpool not available";

/// The ThreadPool used by the VoxelRun game.
pub struct StandardThreadPool {
    sender: Option<mpsc::Sender<Job>>,
    workers: Vec<Worker>,
}

/// execute a function on the global threadpool, disgarding any handle and return value
pub fn global_exec<F>(f: F)
where
    F: FnOnce() + Send + 'static,
{
    if let Some(pool) = unsafe { &THREADPOOL } {
        pool.execute(f)
    } else {
        error!(target: "ThreadPool", "{NO_THREADPOOL_ERROR}")
    }
}

/// execute a function on a global threadpool and returns a [Promise].
pub fn global_promise<F, T>(f: F) -> Promise<T>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    if let Some(pool) = unsafe { &THREADPOOL } {
        pool.promised(f)
    } else {
        error!(target: "ThreadPool", "{NO_THREADPOOL_ERROR}");
        let (_, receiver) = mpsc::channel();
        receiver.into()
    }
}

/// initialize the global threadpool
pub fn global_init() {
    THREADPOOL_INIT.call_once(|| unsafe {
        let _ = &*THREADPOOL.get_or_insert(Arc::new(StandardThreadPool::new(
            available_parallelism().unwrap(),
        )));
    });
}

static mut THREADPOOL: Option<Arc<StandardThreadPool>> = None;
static THREADPOOL_INIT: Once = Once::new();

// insert a cleanup function of the global threadpool in the cleanup section of the binary
#[doc(hidden)]
#[allow(non_upper_case_globals)]
#[used]
#[cfg_attr(target_os = "linux", link_section = ".fini_array.65535")]
#[cfg_attr(target_os = "windows", link_section = ".CRT$XPTZ65535")]
#[cfg_attr(target_os = "macos", link_section = "__DATA,__mod_term_func")]
pub static __static_init_destructor: extern "C" fn() = {
    extern "C" fn __drop_global_pool() {
        if let Some(pool) = unsafe { THREADPOOL.take() } {
            drop(pool)
        }
    }
    __drop_global_pool
};

impl ThreadPool for StandardThreadPool {
    fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        if let Err(_) = self.sender.as_ref().unwrap().send(job) {
            error!(target: "ThreadPool", "{NO_THREADPOOL_ERROR}")
        };
    }

    fn promised<F, T>(&self, f: F) -> Promise<T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        let (sender, receiver) = mpsc::channel();
        let job = Box::new(move || {
            let result = f();
            sender.send(result).expect("Failed to send job");
        });

        if let Err(_) = self.sender.as_ref().unwrap().send(job) {
            error!(target: "ThreadPool", "{NO_THREADPOOL_ERROR}")
        };

        receiver.into()
    }
}

impl StandardThreadPool {
    /// creates a new threadpool with the given sice number of worker threads
    pub fn new(size: NonZeroUsize) -> Self {
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size.into());
        workers.reserve(size.into());

        for id in 0..size.into() {
            workers.push(Worker::new(id, receiver.clone()));
        }

        StandardThreadPool {
            sender: Some(sender),
            workers,
        }
    }
}

/// One worker in a threadpool
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    /// spawns a new worker that listens to a receiver for tasks
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Self {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();
            match message {
                Ok(job) => {
                    debug!(target: "ThreadPool", "Worker {id} got a job; executing");
                    job()
                }
                Err(_) => {
                    debug!(target: "ThreadPool", "Worker {id} disconnected; shutting down");
                    break;
                }
            }
        });
        Worker {
            id,
            thread: Some(thread),
        }
    }
}

impl Drop for StandardThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in self.workers.iter_mut() {
            info!(target: "ThreadPool", "Shutting down worker {}", worker.id);
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}
