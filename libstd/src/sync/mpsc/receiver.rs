use alloc::arc::Arc;
use super::mpsc_queue::PopResult::*;
use super::mpsc_queue::{Queue};

use thread;

pub enum TryRecvError {
    Empty,
    Disconnected,
}

pub struct Receiver<T> {
    pub queue: Arc<Queue<T>>,
}

impl<T> Receiver<T> {
    pub fn try_recv(&self) -> Result<T, TryRecvError> {
        match self.queue.pop() {
            Data(t) => Ok(t),
            _ => Err(TryRecvError::Empty),
        }
    }

    pub fn recv(&self) -> Result<T, ()> {
        loop {
            match self.queue.pop() {
                Data(t) => return Ok(t),
                _ => thread::yield_now(),
            }
        }

    }
}

unsafe impl<T: Send> Send for Receiver<T> {}
