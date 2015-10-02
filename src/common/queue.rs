use common::vec::*;

//A FIFO Queue
pub struct Queue<T> {
    pub vec: Vec<T>,
}

impl<T> Queue<T> {
    /// Create new queue
    pub fn new() -> Queue<T> {
        Queue { vec: Vec::new() }
    }

    /// Push element to queue
    pub fn push(&mut self, value: T) {
        self.vec.push(value);
    }

    /// Pop the last element
    pub fn pop(&mut self) -> Option<T> {
        self.vec.remove(0)
    }

    /// Get the length of the queue
    pub fn len(&self) -> usize {
        self.vec.len()
    }
}
