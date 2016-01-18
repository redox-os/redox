use super::mpsc_queue::{Queue};
use alloc::arc::Arc;
use super::mpsc_queue::PopResult::*;

pub enum TryRecvError {
    Empty,
    Disconnected,
}

pub struct Receiver<T> {
    pub queue: Arc<Queue<T>>,
}

impl<T> Receiver<T> {
    fn try_recv(&self, t: T) -> Result<T, TryRecvError> {
        match self.queue.pop() {
            Data(t) => Ok(t),
            _ => Err(TryRecvError::Empty),
        }
    }

    fn recv(&self, t: T) -> Result<T, ()> {
        loop {
            match self.queue.pop() {
                Data(t) => return Ok(t),
                _ => continue,
            }
        }

    }
}

unsafe impl<T: Send> Send for Receiver<T> {}
