use core::cmp::Ordering;
use core::ops::{Add, Sub};

use arch::context;

pub const NANOS_PER_MICRO: i32 = 1000;
pub const NANOS_PER_MILLI: i32 = 1000000;
pub const NANOS_PER_SEC: i32 = 1000000000;

/// A duration
#[derive(Copy, Clone)]
pub struct Duration {
    /// The seconds
    pub secs: i64,
    /// The nano seconds
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

    /// Get the current duration
    pub fn monotonic() -> Self {
        ::env().clock_monotonic.lock().clone()
    }

    /// Get the realtime
    pub fn realtime() -> Self {
        ::env().clock_realtime.lock().clone()
    }

    /// Sleep the duration
    pub fn sleep(&self) {
        let start_time = Duration::monotonic();
        loop {
            let elapsed = Duration::monotonic() - start_time;
            if elapsed > *self {
                break;
            } else {
                unsafe { context::context_switch(false) };
            }
        }
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
