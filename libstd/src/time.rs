//! A module for time

use core::cmp::{Ordering, PartialEq};
use core::ops::{Add, Sub};

use system::syscall::{sys_clock_gettime, CLOCK_REALTIME, CLOCK_MONOTONIC, TimeSpec};

pub const NANOS_PER_MICRO: i32 = 1_000;
pub const NANOS_PER_MILLI: i32 = 1_000_000;
pub const NANOS_PER_SEC: i32 = 1_000_000_000;

#[derive(Copy, Clone)]
pub struct Duration {
    pub secs: i64,
    pub nanos: i32,
}

impl Duration {
    /// Create a new duration
    pub fn new(mut secs: i64, mut nanos: i32) -> Self {
        while nanos >= NANOS_PER_SEC || (nanos > 0 && secs < 0) {
            secs += 1;
            nanos -= NANOS_PER_SEC;
        }

        while nanos < 0 && secs > 0 {
            secs -= 1;
            nanos += NANOS_PER_SEC;
        }

        Duration {
            secs: secs,
            nanos: nanos,
        }
    }

    /// Get the monotonic time
    pub fn monotonic() -> Self {
        let mut tp = TimeSpec {
            tv_sec: 0,
            tv_nsec: 0,
        };

        unsafe { sys_clock_gettime(CLOCK_MONOTONIC, &mut tp) };

        Duration::new(tp.tv_sec, tp.tv_nsec)
    }

    /// Get the realtime
    pub fn realtime() -> Self {
        let mut tp = TimeSpec {
            tv_sec: 0,
            tv_nsec: 0,
        };

        unsafe { sys_clock_gettime(CLOCK_REALTIME, &mut tp) };

        Duration::new(tp.tv_sec, tp.tv_nsec)
    }
}

impl Add for Duration {
    type Output = Duration;

    fn add(self, other: Self) -> Self {
        Duration::new(self.secs + other.secs, self.nanos + other.nanos)
    }
}

impl Sub for Duration {
    type Output = Duration;

    fn sub(self, other: Self) -> Self {
        Duration::new(self.secs - other.secs, self.nanos - other.nanos)
    }
}

impl PartialEq for Duration {
    fn eq(&self, other: &Self) -> bool {
        let dif = *self - *other;
        dif.secs == 0 && dif.nanos == 0
    }
}

impl PartialOrd for Duration {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let dif = *self - *other;
        if dif.secs > 0 {
            Some(Ordering::Greater)
        } else if dif.secs < 0 {
            Some(Ordering::Less)
        } else if dif.nanos > 0 {
            Some(Ordering::Greater)
        } else if dif.nanos < 0 {
            Some(Ordering::Less)
        } else {
            Some(Ordering::Equal)
        }
    }
}
