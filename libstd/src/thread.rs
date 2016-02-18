use alloc::boxed::Box;

use system::syscall::{sys_clone, sys_exit, sys_yield, sys_nanosleep, CLONE_VM, CLONE_FS, CLONE_FILES,
              TimeSpec};

use time::Duration;

// TODO: Mutex the result
pub struct JoinHandle<T> {
    result_ptr: *mut Option<T>,
}

impl<T> JoinHandle<T> {
    pub fn join(self) -> Option<T> {
        unsafe {
            while (*self.result_ptr).is_none() {
                let _ = sys_yield();
            }

            *Box::from_raw(self.result_ptr)
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

    // Duration::new(rem.tv_sec, rem.tv_nsec)
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
    unsafe {
        let result_ptr: *mut Option<T> = Box::into_raw(box None);

        if sys_clone(CLONE_VM | CLONE_FS | CLONE_FILES).unwrap() == 0 {
            *result_ptr = Some(f());
            loop {
                let _ = sys_exit(0);
            }
        }

        JoinHandle { result_ptr: result_ptr }
    }
}

pub fn yield_now() {
    let _ = sys_yield();
}
