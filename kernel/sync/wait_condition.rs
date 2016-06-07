use arch::context::{Context, context_switch};

use collections::Vec;

use core::mem;
use core::ops::DerefMut;

use super::Intex;

pub struct WaitCondition {
    contexts: Intex<Vec<*mut Context>>,
}

impl WaitCondition {
    pub fn new() -> WaitCondition {
        WaitCondition { contexts: Intex::new(Vec::new()) }
    }

    pub unsafe fn notify(&self) {
        let mut contexts = Vec::new();
        mem::swap(self.contexts.lock().deref_mut(), &mut contexts);
        for &context in contexts.iter() {
            (*context).blocked = false;
        }
    }

    pub unsafe fn wait(&self) {
        if let Ok(mut context) = ::env().contexts.lock().current_mut() {
            let mut contexts = self.contexts.lock();
            contexts.push(context.deref_mut() as *mut Context);
            (*context).blocked = true;
        }
        context_switch();
    }
}

impl Drop for WaitCondition {
    fn drop(&mut self) {
        unsafe {
            self.notify();
        }
    }
}
