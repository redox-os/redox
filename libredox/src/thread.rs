use alloc::boxed::Box;

use syscall::{sys_clone, sys_exit, sys_yield};
use syscall::common::{CLONE_VM, CLONE_FS, CLONE_FILES};

// TODO: Mutex the result
pub struct JoinHandle<T> {
    result_ptr: *mut Option<T>,
}

impl<T> JoinHandle<T> {
    pub fn join(self) -> Option<T> {
        unsafe {
            while (*self.result_ptr).is_none() {
                sys_yield();
            }

            *Box::from_raw(self.result_ptr)
        }
    }
}

// TODO: Catch panic
pub fn spawn<F, T>(f: F) -> JoinHandle<T>
    where F: FnOnce() -> T,
          F: Send + 'static,
          T: Send + 'static
{
    unsafe {
        let result_ptr: *mut Option<T> = Box::into_raw(box None);

        if sys_clone(CLONE_VM | CLONE_FS | CLONE_FILES) == 0 {
            *result_ptr = Some(f());
            sys_exit(0);
        }

        JoinHandle { result_ptr: result_ptr }
    }
}
