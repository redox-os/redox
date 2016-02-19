use arch::context::{context_switch, Context};

use collections::vec_deque::VecDeque;

use core::ops::DerefMut;

use super::Intex;

pub struct WaitCondition {
    contexts: Intex<VecDeque<*mut Context>>
}

impl WaitCondition {
    pub fn new() -> WaitCondition {
        WaitCondition {
            contexts: Intex::new(VecDeque::new())
        }
    }

    pub unsafe fn notify(&self) {
        let mut contexts = self.contexts.lock();
        while let Some(context) = contexts.pop_front() {
            (*context).blocked = false;
        }
    }

    pub unsafe fn wait(&self) {
        if let Ok(mut context) = ::env().contexts.lock().current_mut() {
            let mut contexts = self.contexts.lock();
            contexts.push_back(context.deref_mut() as *mut Context);
            (*context).blocked = true;
        }
        context_switch();
    }
}

impl Drop for WaitCondition {
    fn drop(&mut self){
        unsafe { self.notify(); }
    }
}
