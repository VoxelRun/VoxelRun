use std::sync::mpsc::{self, RecvError};

pub struct Promise<T> {
    indicator: mpsc::Receiver<T>
}


impl<T> From<mpsc::Receiver<T>> for Promise<T>{
    fn from(value: mpsc::Receiver<T>) -> Self {
        Self { indicator: value }
    }
}

impl<T> Promise<T> {
    pub fn get(&self) -> T {
        self.indicator.recv().unwrap()
    }
    pub fn try_get(&self) -> Result<T, RecvError> {
        self.indicator.recv()
    }
}
