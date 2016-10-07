use arch;
use context;
use syscall::data::TimeSpec;
use syscall::error::*;
use syscall::flag::{CLOCK_REALTIME, CLOCK_MONOTONIC};

pub fn clock_gettime(clock: usize, time: &mut TimeSpec) -> Result<usize> {
    match clock {
        CLOCK_REALTIME => {
            let arch_time = arch::time::realtime();
            time.tv_sec = arch_time.0 as i64;
            time.tv_nsec = arch_time.1 as i32;
            Ok(0)
        },
        CLOCK_MONOTONIC => {
            let arch_time = arch::time::monotonic();
            time.tv_sec = arch_time.0 as i64;
            time.tv_nsec = arch_time.1 as i32;
            Ok(0)
        },
        _ => Err(Error::new(EINVAL))
    }
}

pub fn nanosleep(req: &TimeSpec, rem_opt: Option<&mut TimeSpec>) -> Result<usize> {
    let start = arch::time::monotonic();
    let sum = start.1 + req.tv_nsec as u64;
    let end = (start.0 + req.tv_sec as u64 + sum / 1000000000, sum % 1000000000);
    
    loop {
        unsafe { context::switch(); }

        let current = arch::time::monotonic();
        if current.0 > end.0 || (current.0 == end.0 && current.1 >= end.1) {
            break;
        }
    }

    if let Some(mut rem) = rem_opt {
        rem.tv_sec = 0;
        rem.tv_nsec = 0;
    }

    Ok(0)
}

pub fn sched_yield() -> Result<usize> {
    unsafe { context::switch(); }
    Ok(0)
}
