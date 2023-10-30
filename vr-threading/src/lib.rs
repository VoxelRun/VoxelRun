use std::{
    num::NonZeroUsize,
    sync::{mpsc, Arc, Mutex},
    thread::{self, available_parallelism},
};

use global_threadpool::ThreadPool;
use lazy_static::lazy_static;

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct StandardThreadPool {
    sender: Option<mpsc::Sender<Job>>,
    workers: Vec<Worker>,
}

pub mod global_threadpool;

lazy_static! {
    pub static ref THREADPOOL: StandardThreadPool =
        StandardThreadPool::new(available_parallelism().unwrap());
}

impl ThreadPool for StandardThreadPool {
    fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap();
    }

    fn execute_eval<F, T>(&self, f: F) -> mpsc::Receiver<T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        let (sender, receiver) = mpsc::channel();
        let job = Box::new(move || {
            let result = f();
            sender.send(result).expect("Failed to send job");
        });

        self.sender.as_ref().unwrap().send(job).unwrap();

        receiver
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
                    println!("ThreadPool: Worker {id} got a job; executing");
                    job()
                }
                Err(_) => {
                    println!("ThreadPool: Worker {id} disconnected; shutting down");
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
            println!("ThreadPool: Shutting down worker {}", worker.id);
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}
