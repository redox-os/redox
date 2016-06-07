//! System calles related to time.

use arch::context::context_switch;

use common::time::Duration;

use syscall::{CLOCK_MONOTONIC, CLOCK_REALTIME, TimeSpec};

use system::error::{Error, Result, EINVAL};

/// Get the time of a given clock.
pub fn clock_gettime(clock: usize, tp: *mut TimeSpec) -> Result<usize> {
    let contexts = ::env().contexts.lock();
    let current = try!(contexts.current());
    let tp_safe = try!(current.safe_ref_mut(tp));

    match clock {
        CLOCK_REALTIME => {
            let clock_realtime = ::env().clock_realtime.lock();
            tp_safe.tv_sec = clock_realtime.secs;
            tp_safe.tv_nsec = clock_realtime.nanos;
            Ok(0)
        }
        CLOCK_MONOTONIC => {
            let clock_monotonic = ::env().clock_monotonic.lock();
            tp_safe.tv_sec = clock_monotonic.secs;
            tp_safe.tv_nsec = clock_monotonic.nanos;
            Ok(0)
        }
        _ => Err(Error::new(EINVAL)),
    }
}

/// Sleep in N nanoseconds.
pub fn nanosleep(req: *const TimeSpec, rem: *mut TimeSpec) -> Result<usize> {
    {
        let mut contexts = ::env().contexts.lock();
        let mut current = try!(contexts.current_mut());

        // Copied with * to avoid borrow issue on current.blocked = true
        let req_safe = *try!(current.safe_ref(req));

        current.blocked = true;
        current.wake = Some(Duration::monotonic() + Duration::new(req_safe.tv_sec, req_safe.tv_nsec));
    }

    unsafe { context_switch(); }

    {
        let contexts = ::env().contexts.lock();
        let current = try!(contexts.current());

        if let Ok(rem_safe) = current.safe_ref_mut(rem) {
            rem_safe.tv_sec = 0;
            rem_safe.tv_nsec = 0;
        }
    }

    Ok(0)
}
