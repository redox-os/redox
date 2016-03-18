//! A module for time

use core::cmp::{Ordering, PartialEq};
use core::ops::{Add, Sub};

use system::syscall::{sys_clock_gettime, CLOCK_REALTIME, CLOCK_MONOTONIC, TimeSpec};

pub const NANOS_PER_MICRO: i32 = 1_000;
pub const NANOS_PER_MILLI: i32 = 1_000_000;
pub const NANOS_PER_SEC: i32 = 1_000_000_000;

/// A duration type to represent a span of time, typically used for system
/// timeouts.
///
/// Each duration is composed of a number of seconds and nanosecond precision.
/// APIs binding a system timeout will typically round up the nanosecond
/// precision if the underlying system does not support that level of precision.
///
/// Durations implement many common traits, including `Add`, `Sub`, and other
/// ops traits. Currently a duration may only be inspected for its number of
/// seconds and its nanosecond precision.
///
/// # Examples
///
/// ```
/// use std::time::Duration;
///
/// let five_seconds = Duration::new(5, 0);
/// let five_seconds_and_five_nanos = five_seconds + Duration::new(0, 5);
///
/// assert_eq!(five_seconds_and_five_nanos.as_secs(), 5);
/// assert_eq!(five_seconds_and_five_nanos.subsec_nanos(), 5);
///
/// let ten_millis = Duration::from_millis(10);
/// ```
#[derive(Copy, Clone)]
pub struct Duration {
    pub secs: i64,
    pub nanos: i32,
}

impl Duration {
    /// Creates a new `Duration` from the specified number of seconds and
    /// additional nanosecond precision.
    ///
    /// If the nanoseconds is greater than 1 billion (the number of nanoseconds
    /// in a second), then it will carry over into the seconds provided.
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

    /// Creates a new `Duration` from the specified number of milliseconds.
    pub fn from_millis(millis: u64) -> Self {
        Duration::new((millis / 1000) as i64, (millis % 1000) as i32 * NANOS_PER_MILLI)
    }

    pub fn as_secs(&self) -> u64 {
        self.secs as u64
    }

    pub fn subsec_nanos(&self) -> u32 {
        self.nanos as u32
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

#[derive(Copy, Clone)]
pub struct Instant(Duration);

impl Instant {
    /// Returns an instant corresponding to "now".
    pub fn now() -> Instant {
        let mut tp = TimeSpec {
            tv_sec: 0,
            tv_nsec: 0,
        };

        sys_clock_gettime(CLOCK_MONOTONIC, &mut tp).unwrap();

        Instant(Duration::new(tp.tv_sec, tp.tv_nsec))
    }

    /// Returns the amount of time between two instants
    pub fn duration_since(&self, earlier: Instant) -> Duration {
        self.0 - earlier.0
    }

    /// Returns the amount of time elapsed since this instant was created.
    ///
    /// # Panics
    ///
    /// This function may panic if the current time is earlier than this
    /// instant, which is something that can happen if an `Instant` is
    /// produced synthetically.
    pub fn elapsed(&self) -> Duration {
        Instant::now().0 - self.0
    }
}

#[derive(Copy, Clone)]
pub struct SystemTime(Duration);

impl SystemTime {
    /// Returns the system time corresponding to "now".
    pub fn now() -> SystemTime {
        let mut tp = TimeSpec {
            tv_sec: 0,
            tv_nsec: 0,
        };

        sys_clock_gettime(CLOCK_REALTIME, &mut tp).unwrap();

        SystemTime(Duration::new(tp.tv_sec, tp.tv_nsec))
    }
}
