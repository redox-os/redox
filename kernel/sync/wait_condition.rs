use arch::context::{context_switch, Context};

use collections::Vec;

use core::cell::UnsafeCell;
use core::mem;
use core::ops::DerefMut;

pub struct WaitCondition {
    contexts: UnsafeCell<Vec<*mut Context>>
}

impl WaitCondition {
    pub fn new() -> WaitCondition {
        WaitCondition {
            contexts: UnsafeCell::new(Vec::new())
        }
    }

    pub fn notify(&self) {
        let mut contexts = Vec::new();
        mem::swap(unsafe { &mut *self.contexts.get() }, &mut contexts);
        for &context in contexts.iter() {
            unsafe { (*context).blocked = false; }
        }
    }

    pub fn wait(&self) {
        if let Ok(mut context) = unsafe { &mut *::env().contexts.get() }.current_mut() {
            let mut contexts = unsafe { &mut *self.contexts.get() };
            contexts.push(context.deref_mut() as *mut Context);
            (*context).blocked = true;
        }
        unsafe { context_switch(); }
    }
}

impl Drop for WaitCondition {
    fn drop(&mut self){
        self.notify();
    }
}
