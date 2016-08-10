use cell::UnsafeCell;
use intrinsics;
use i32;

use system::syscall::{sys_futex, FUTEX_WAIT, FUTEX_WAKE};

pub struct Futex(UnsafeCell<i32>);

impl Futex {
    pub fn new() -> Futex {
        Futex(UnsafeCell::new(0))
    }

    pub fn wait(&self) {
        if unsafe { intrinsics::atomic_xsub(self.0.get(), 1) } == 1 {
            unsafe { intrinsics::atomic_store(self.0.get(), -1) };
            sys_futex(unsafe { &mut *self.0.get() }, FUTEX_WAIT, -1).unwrap();
        }
    }

    pub fn wake(&self) -> usize {
        if unsafe { intrinsics::atomic_xchg(self.0.get(), 1) } < 0 {
            sys_futex(unsafe { &mut *self.0.get() }, FUTEX_WAKE, i32::MAX).unwrap()
        } else {
            0
        }
    }
}

unsafe impl Send for Futex {}

unsafe impl Sync for Futex {}
