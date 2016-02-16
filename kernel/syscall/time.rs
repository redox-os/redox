use common::time::Duration;

use super::{Error, Result, CLOCK_MONOTONIC, CLOCK_REALTIME, EFAULT, EINVAL, TimeSpec};

pub fn do_sys_clock_gettime(clock: usize, tp: *mut TimeSpec) -> Result<usize> {
    if tp as usize > 0 {
        match clock {
            CLOCK_REALTIME => {
                let clock_realtime = ::env().clock_realtime.lock();
                unsafe {
                    (*tp).tv_sec = clock_realtime.secs;
                    (*tp).tv_nsec = clock_realtime.nanos;
                }
                Ok(0)
            }
            CLOCK_MONOTONIC => {
                let clock_monotonic = ::env().clock_monotonic.lock();
                unsafe {
                    (*tp).tv_sec = clock_monotonic.secs;
                    (*tp).tv_nsec = clock_monotonic.nanos;
                }
                Ok(0)
            }
            _ => Err(Error::new(EINVAL)),
        }
    } else {
        Err(Error::new(EFAULT))
    }
}

pub fn do_sys_nanosleep(req: *const TimeSpec, rem: *mut TimeSpec) -> Result<usize> {
    if req as usize > 0 {
        Duration::new(unsafe { (*req).tv_sec }, unsafe { (*req).tv_nsec }).sleep();

        if rem as usize > 0 {
            unsafe {
                (*rem).tv_sec = 0;
            }
            unsafe {
                (*rem).tv_nsec = 0;
            }
        }

        Ok(0)
    } else {
        Err(Error::new(EFAULT))
    }
}
