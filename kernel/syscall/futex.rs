use alloc::arc::Arc;
use collections::VecDeque;
use core::intrinsics;
use spin::{Once, RwLock, RwLockReadGuard, RwLockWriteGuard};

use context::{self, Context};
use syscall::error::{Error, Result, ESRCH, EAGAIN, EINVAL};
use syscall::flag::{FUTEX_WAIT, FUTEX_WAKE, FUTEX_REQUEUE};
use syscall::validate::validate_slice_mut;

type FutexList = VecDeque<(usize, Arc<RwLock<Context>>)>;

/// Fast userspace mutex list
static FUTEXES: Once<RwLock<FutexList>> = Once::new();

/// Initialize futexes, called if needed
fn init_futexes() -> RwLock<FutexList> {
    RwLock::new(VecDeque::new())
}

/// Get the global futexes list, const
pub fn futexes() -> RwLockReadGuard<'static, FutexList> {
    FUTEXES.call_once(init_futexes).read()
}

/// Get the global futexes list, mutable
pub fn futexes_mut() -> RwLockWriteGuard<'static, FutexList> {
    FUTEXES.call_once(init_futexes).write()
}

pub fn futex(addr: &mut i32, op: usize, val: i32, val2: usize, addr2: *mut i32) -> Result<usize> {
    match op {
        FUTEX_WAIT => {
            {
                let mut futexes = futexes_mut();

                let context_lock = {
                    let contexts = context::contexts();
                    let context_lock = contexts.current().ok_or(Error::new(ESRCH))?;
                    context_lock.clone()
                };

                if unsafe { intrinsics::atomic_load(addr) != val } {
                    return Err(Error::new(EAGAIN));
                }

                context_lock.write().block();

                futexes.push_back((addr as *mut i32 as usize, context_lock));
            }

            unsafe { context::switch(); }

            Ok(0)
        },
        FUTEX_WAKE => {
            let mut woken = 0;

            {
                let mut futexes = futexes_mut();

                let mut i = 0;
                while i < futexes.len() && (woken as i32) < val {
                    if futexes[i].0 == addr as *mut i32 as usize {
                        if let Some(futex) = futexes.swap_remove_back(i) {
                            futex.1.write().unblock();
                            woken += 1;
                        }
                    } else {
                        i += 1;
                    }
                }
            }

            Ok(woken)
        },
        FUTEX_REQUEUE => {
            let addr2_safe = validate_slice_mut(addr2, 1).map(|addr2_safe| &mut addr2_safe[0])?;

            let mut woken = 0;
            let mut requeued = 0;

            {
                let mut futexes = futexes_mut();

                let mut i = 0;
                while i < futexes.len() && (woken as i32) < val {
                    if futexes[i].0 == addr as *mut i32 as usize {
                        if let Some(futex) = futexes.swap_remove_back(i) {
                            futex.1.write().unblock();
                            woken += 1;
                        }
                    } else {
                        i += 1;
                    }
                }
                while i < futexes.len() && requeued < val2 {
                    if futexes[i].0 == addr as *mut i32 as usize {
                        futexes[i].0 = addr2_safe as *mut i32 as usize;
                        requeued += 1;
                    }
                    i += 1;
                }
            }

            Ok(woken)
        },
        _ => Err(Error::new(EINVAL))
    }
}
