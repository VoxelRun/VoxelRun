use std::{
    num::NonZeroUsize,
    sync::{
        mpsc::{self, Receiver},
        Arc, Mutex, Once,
    },
    thread::{self, available_parallelism},
};

use global_threadpool::ThreadPool;
use promises::Promise;
use vr_logger::{debug, error, info};

pub mod global_threadpool;
pub mod promises;

type Job = Box<dyn FnOnce() + Send + 'static>;

const NO_THREADPOOL_ERROR: &str = "Tried to execute a task while threadpool not available";

pub struct StandardThreadPool {
    sender: Option<mpsc::Sender<Job>>,
    workers: Vec<Worker>,
}

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

pub fn global_init() {
    THREADPOOL_INIT.call_once(|| unsafe {
        let _ = &*THREADPOOL.get_or_insert(Arc::new(StandardThreadPool::new(
            available_parallelism().unwrap(),
        )));
    });
}

static mut THREADPOOL: Option<Arc<StandardThreadPool>> = None;
static THREADPOOL_INIT: Once = Once::new();

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

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
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
