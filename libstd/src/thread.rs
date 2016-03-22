use alloc::boxed::Box;

use core::mem;

use system::syscall::{sys_clone, sys_exit, sys_yield, sys_nanosleep, sys_waitpid, CLONE_VM, CLONE_FS, CLONE_FILES,
              TimeSpec};

use time::Duration;

/// An owned permission to join on a thread (block on its termination).
///
/// A `JoinHandle` *detaches* the child thread when it is dropped.
///
/// Due to platform restrictions, it is not possible to `Clone` this
/// handle: the ability to join a child thread is a uniquely-owned
/// permission.
// TODO: Mutex the result
pub struct JoinHandle<T> {
    pid: usize,
    result_ptr: *mut Option<T>,
}

impl<T> JoinHandle<T> {
    /// Waits for the associated thread to finish.
    pub fn join(self) -> Option<T> where T: ::core::fmt::Debug {
        let mut status = 0;
        match sys_waitpid(self.pid, &mut status, 0) {
            Ok(_) => unsafe { *Box::from_raw(self.result_ptr) },
            Err(_) => None
        }
    }
}

/// Sleep for a duration
pub fn sleep(duration: Duration) {
    let req = TimeSpec {
        tv_sec: duration.as_secs() as i64,
        tv_nsec: duration.subsec_nanos() as i32,
    };

    let mut rem = TimeSpec {
        tv_sec: 0,
        tv_nsec: 0,
    };

    let _ = sys_nanosleep(&req, &mut rem);
}

/// Sleep for a number of milliseconds
pub fn sleep_ms(ms: u32) {
    let secs = ms as u64 / 1000;
    let nanos = (ms % 1000) * 1000000;
    sleep(Duration::new(secs, nanos))
}

/// Spawns a new thread, returning a `JoinHandle` for it.
///
/// The join handle will implicitly *detach* the child thread upon being
/// dropped. In this case, the child thread may outlive the parent (unless
/// the parent thread is the main thread; the whole process is terminated when
/// the main thread finishes.) Additionally, the join handle provides a `join`
/// method that can be used to join the child thread. If the child thread
/// panics, `join` will return an `Err` containing the argument given to
/// `panic`.
///
/// # Panics
///
/// Panics if the OS fails to create a thread; use `Builder::spawn`
/// to recover from such errors.
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
