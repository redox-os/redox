use super::mpsc_queue::{Queue};

pub struct Receiver<T> {
    pub queue: *const Queue<T>,
}

impl<T> Receiver<T> {
    fn recv(&self, t: T) -> Result<T, ()> {
        use super::mpsc_queue::PopResult::*;

        loop {
            match unsafe { (*self.queue).pop() } {
                Data(t) => return Ok(t),
                _ => continue,
            }
        }

    }
}

unsafe impl<T: Send> Send for Receiver<T> {}
