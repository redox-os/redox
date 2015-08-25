use common::vec::*;

use programs::common::sched_yield;

//A FIFO Queue
pub struct Queue<T> {
    vec: Vec<T>
}

impl<T> Queue<T> {
    pub fn new() -> Queue<T> {
        Queue {
            vec: Vec::new()
        }
    }

    pub fn push(&mut self, value: T){
        self.vec.push(value);
    }

    pub fn pop(&mut self) -> Option<T> {
        return self.vec.remove(0);
    }

    pub fn wait(&mut self) -> T {
        loop {
            match self.pop() {
                Option::Some(value) => return value,
                Option::None => sched_yield()
            }
        }
    }
}
