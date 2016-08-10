use arch::context::{context_switch, Context};

use collections::Vec;

use core::cell::UnsafeCell;
use core::mem;
use core::ops::DerefMut;

use common::time::Duration;

pub struct WaitCondition {
    contexts: UnsafeCell<Vec<*mut Context>>
}

impl WaitCondition {
    pub fn new() -> WaitCondition {
        WaitCondition {
            contexts: UnsafeCell::new(Vec::new())
        }
    }

    pub fn notify(&self, reason: &str) -> usize {
        /*{
            debugln!("  WaitCondition::notify {:X} {}", self as *const _ as usize, reason);
            if let Ok(context) = unsafe { &mut *::env().contexts.get() }.current_mut() {
                debugln!("    FROM {}: {}", (*context).pid, (*context).name);
            } else {
                debugln!("    NOT FOUND {}/{}", unsafe { & *::env().contexts.get() }.i, unsafe { & *::env().contexts.get() }.len());
            }
        }*/
        let mut contexts = Vec::new();
        mem::swap(unsafe { &mut *self.contexts.get() }, &mut contexts);
        for &context in contexts.iter() {
            unsafe { (*context).unblock(reason) }
        }
        contexts.len()
    }

    pub fn wait(&self, reason: &str) {
        {
            // debugln!("  WaitCondition::wait {:X} {}", self as *const _ as usize, reason);
            if let Ok(mut context) = unsafe { &mut *::env().contexts.get() }.current_mut() {
                let mut contexts = unsafe { &mut *self.contexts.get() };
                contexts.push(context.deref_mut() as *mut Context);
                (*context).block(reason);
            } else {
                // debugln!("    NOT FOUND {}/{}", unsafe { & *::env().contexts.get() }.i, unsafe { & *::env().contexts.get() }.len());
            }
        }
        unsafe { context_switch(); }
    }

    pub fn wait_for(&self, reason: &str, time: Duration) -> bool {
        let mut ret = true;
        {
            // debugln!("  WaitCondition::wait_until {:X} {}", self as *const _ as usize, reason);
            if let Ok(mut context) = unsafe { &mut *::env().contexts.get() }.current_mut() {
                let mut contexts = unsafe { &mut *self.contexts.get() };
                contexts.push(context.deref_mut() as *mut Context);
                (*context).wake = Some(Duration::monotonic() + time);
                (*context).block(reason);
            } else {
                // debugln!("    NOT FOUND {}/{}", unsafe { & *::env().contexts.get() }.i, unsafe { & *::env().contexts.get() }.len());
            }
        }
        unsafe { context_switch(); }
        {
            if let Ok(mut context) = unsafe { &mut *::env().contexts.get() }.current_mut() {
                if (*context).wake.is_none() {
                    ret = false;
                } else {
                    (*context).wake = None;
                }
            }
        }
        ret
    }
}

impl Drop for WaitCondition {
    fn drop(&mut self){
        self.notify("WaitCondition::drop");
    }
}
