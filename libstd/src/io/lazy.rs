// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use prelude::v1::*;

use cell::Cell;
use ptr;
use sync::{StaticMutex, Arc};

pub struct Lazy<T> {
    lock: StaticMutex,
    ptr: Cell<*mut Arc<T>>,
    init: fn() -> Arc<T>,
}

unsafe impl<T> Sync for Lazy<T> {}

impl<T: Send + Sync + 'static> Lazy<T> {
    pub const fn new(init: fn() -> Arc<T>) -> Lazy<T> {
        Lazy {
            lock: StaticMutex::new(),
            ptr: Cell::new(ptr::null_mut()),
            init: init
        }
    }

    pub fn get(&'static self) -> Option<Arc<T>> {
        let _g = self.lock.lock();
        let ptr = self.ptr.get();
        unsafe {
            if ptr.is_null() {
                Some(self.init())
            } else if ptr as usize == 1 {
                None
            } else {
                Some((*ptr).clone())
            }
        }
    }

    unsafe fn init(&'static self) -> Arc<T> {
        (self.init)()
    }
}
