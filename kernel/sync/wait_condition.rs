use alloc::arc::Arc;
use collections::Vec;
use spin::{Mutex, RwLock};

use context::{self, Context};

#[derive(Debug)]
pub struct WaitCondition {
    contexts: Mutex<Vec<Arc<RwLock<Context>>>>
}

impl WaitCondition {
    pub fn new() -> WaitCondition {
        WaitCondition {
            contexts: Mutex::new(Vec::with_capacity(16))
        }
    }

    pub fn notify(&self) -> usize {
        let mut contexts = self.contexts.lock();
        let len = contexts.len();
        while let Some(context_lock) = contexts.pop() {
            context_lock.write().unblock();
        }
        len
    }

    pub fn wait(&self) {
        {
            let context_lock = {
                let contexts = context::contexts();
                let context_lock = contexts.current().expect("WaitCondition::wait: no context");
                context_lock.clone()
            };

            context_lock.write().block();

            self.contexts.lock().push(context_lock);
        }
        unsafe { context::switch(); }
    }
}

impl Drop for WaitCondition {
    fn drop(&mut self){
        self.notify();
    }
}
