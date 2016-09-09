//! System calles related to time.

use arch::context::context_switch;

use common::time::Duration;

use syscall::{CLOCK_MONOTONIC, CLOCK_REALTIME, TimeSpec};

use system::error::{Error, Result, EINVAL};

/// Get the time of a given clock.
pub fn clock_gettime(clock: usize, tp: &mut TimeSpec) -> Result<usize> {
    match clock {
        CLOCK_REALTIME => {
            let clock_realtime = Duration::realtime();
            tp.tv_sec = clock_realtime.secs;
            tp.tv_nsec = clock_realtime.nanos;
            Ok(0)
        }
        CLOCK_MONOTONIC => {
            let clock_monotonic = Duration::monotonic();
            tp.tv_sec = clock_monotonic.secs;
            tp.tv_nsec = clock_monotonic.nanos;
            Ok(0)
        }
        _ => Err(Error::new(EINVAL)),
    }
}

/// Sleep in N nanoseconds.
pub fn nanosleep(req: &TimeSpec, rem: Option<&mut TimeSpec>) -> Result<usize> {
    {
        let contexts = unsafe { &mut *::env().contexts.get() };
        let mut current = try!(contexts.current_mut());

        // Copied with * to avoid borrow issue on current.blocked = true
        let req = *req;

        current.block("nanosleep");
        current.wake = Some(Duration::monotonic() + Duration::new(req.tv_sec, req.tv_nsec));
    }

    unsafe { context_switch(); }

    if let Some(rem) = rem {
        rem.tv_sec = 0;
        rem.tv_nsec = 0;
    }

    Ok(0)
}
