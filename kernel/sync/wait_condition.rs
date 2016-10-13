use alloc::arc::Arc;
use collections::Vec;
use core::mem;
use spin::{Mutex, RwLock};

use context::{self, Context};

#[derive(Debug)]
pub struct WaitCondition {
    contexts: Mutex<Vec<Arc<RwLock<Context>>>>
}

impl WaitCondition {
    pub fn new() -> WaitCondition {
        WaitCondition {
            contexts: Mutex::new(Vec::new())
        }
    }

    pub fn notify(&self) -> usize {
        let mut contexts = Vec::new();
        mem::swap(&mut *self.contexts.lock(), &mut contexts);
        for context_lock in contexts.iter() {
            context_lock.write().unblock();
        }
        contexts.len()
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
