use super::mpsc_queue::Queue;
use alloc::boxed::Box;

pub struct Sender<T> {
    pub queue: Box<Queue<T>>,
}

impl<T> Sender<T> {
    fn send(&self, t: T) -> Result<(), ()> {
        self.queue.push(t);
        Ok(())
    }
}

unsafe impl<T: Send> Send for Sender<T> {}
