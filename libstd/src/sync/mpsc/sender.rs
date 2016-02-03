use super::mpsc_queue::Queue;
use alloc::arc::Arc;

#[derive(Clone)]
pub struct Sender<T> {
    pub queue: Arc<Queue<T>>,//Box<Queue<T>>,
}

impl<T> Sender<T> {
    pub fn send(&self, t: T) -> Result<(), ()> {
        self.queue.push(t);
        Ok(())
    }
}

unsafe impl<T: Send> Send for Sender<T> {}
