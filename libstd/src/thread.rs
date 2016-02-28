use alloc::boxed::Box;

use core::mem;

use system::syscall::{sys_clone, sys_exit, sys_yield, sys_nanosleep, sys_waitpid, CLONE_VM, CLONE_FS, CLONE_FILES,
              TimeSpec};

use time::Duration;

// TODO: Mutex the result
pub struct JoinHandle<T> {
    pid: usize,
    result_ptr: *mut Option<T>,
}

impl<T> JoinHandle<T> {
    pub fn join(self) -> Option<T> where T: ::core::fmt::Debug {
        let mut status = 0;
        match sys_waitpid(self.pid, &mut status, 0) {
            Ok(_) => unsafe { *Box::from_raw(self.result_ptr) },
            Err(_) => None
        }
    }
}

// Sleep for a duration
pub fn sleep(duration: Duration) {
    let req = TimeSpec {
        tv_sec: duration.secs,
        tv_nsec: duration.nanos,
    };

    let mut rem = TimeSpec {
        tv_sec: 0,
        tv_nsec: 0,
    };

    let _ = sys_nanosleep(&req, &mut rem);
}

// Sleep for a number of milliseconds
pub fn sleep_ms(ms: u32) {
    let secs = ms as i64 / 1000;
    let nanos = (ms % 1000) as i32 * 1000000;
    sleep(Duration::new(secs, nanos))
}

// TODO: Catch panic
pub fn spawn<F, T>(f: F) -> JoinHandle<T>
    where F: FnOnce() -> T,
          F: Send + 'static,
          T: Send + 'static
{
    let result_ptr: *mut Option<T> = Box::into_raw(box None);
    //This must only be used by the child
    let boxed_f = Box::new(f);

    match unsafe { sys_clone(CLONE_VM | CLONE_FS | CLONE_FILES).unwrap() } {
        0 => {
            unsafe { *result_ptr = Some(boxed_f()) };
            loop {
                let _ = sys_exit(0);
            }
        },
        pid => {
            //Forget so that the parent will not drop while the child is using
            mem::forget(boxed_f);
            JoinHandle {
                pid: pid,
                result_ptr: result_ptr
            }
        }
    }
}

pub fn yield_now() {
    let _ = sys_yield();
}
