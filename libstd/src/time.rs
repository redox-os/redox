//! A module for time

use system::syscall::{sys_clock_gettime, CLOCK_REALTIME, CLOCK_MONOTONIC, TimeSpec};

// Copyright 2012-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use ops::{Add, Sub, Mul, Div};

const NANOS_PER_SEC: u32 = 1_000_000_000;
const NANOS_PER_MILLI: u32 = 1_000_000;
const MILLIS_PER_SEC: u64 = 1_000;

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
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct Duration {
    secs: u64,
    nanos: u32, // Always 0 <= nanos < NANOS_PER_SEC
}

impl Duration {
    /// Creates a new `Duration` from the specified number of seconds and
    /// additional nanosecond precision.
    ///
    /// If the nanoseconds is greater than 1 billion (the number of nanoseconds
    /// in a second), then it will carry over into the seconds provided.
    pub fn new(secs: u64, nanos: u32) -> Duration {
        let secs = secs + (nanos / NANOS_PER_SEC) as u64;
        let nanos = nanos % NANOS_PER_SEC;
        Duration { secs: secs, nanos: nanos }
    }

    /// Creates a new `Duration` from the specified number of seconds.
    pub fn from_secs(secs: u64) -> Duration {
        Duration { secs: secs, nanos: 0 }
    }

    /// Creates a new `Duration` from the specified number of milliseconds.
    pub fn from_millis(millis: u64) -> Duration {
        let secs = millis / MILLIS_PER_SEC;
        let nanos = ((millis % MILLIS_PER_SEC) as u32) * NANOS_PER_MILLI;
        Duration { secs: secs, nanos: nanos }
    }

    /// Returns the number of whole seconds represented by this duration.
    ///
    /// The extra precision represented by this duration is ignored (e.g. extra
    /// nanoseconds are not represented in the returned value).
    pub fn as_secs(&self) -> u64 { self.secs }

    /// Returns the nanosecond precision represented by this duration.
    ///
    /// This method does **not** return the length of the duration when
    /// represented by nanoseconds. The returned number always represents a
    /// fractional portion of a second (e.g. it is less than one billion).
    pub fn subsec_nanos(&self) -> u32 { self.nanos }
}

impl Add for Duration {
    type Output = Duration;

    fn add(self, rhs: Duration) -> Duration {
        let mut secs = self.secs.checked_add(rhs.secs)
                           .expect("overflow when adding durations");
        let mut nanos = self.nanos + rhs.nanos;
        if nanos >= NANOS_PER_SEC {
            nanos -= NANOS_PER_SEC;
            secs = secs.checked_add(1).expect("overflow when adding durations");
        }
        debug_assert!(nanos < NANOS_PER_SEC);
        Duration { secs: secs, nanos: nanos }
    }
}

impl Sub for Duration {
    type Output = Duration;

    fn sub(self, rhs: Duration) -> Duration {
        let mut secs = self.secs.checked_sub(rhs.secs)
                           .expect("overflow when subtracting durations");
        let nanos = if self.nanos >= rhs.nanos {
            self.nanos - rhs.nanos
        } else {
            secs = secs.checked_sub(1)
                       .expect("overflow when subtracting durations");
            self.nanos + NANOS_PER_SEC - rhs.nanos
        };
        debug_assert!(nanos < NANOS_PER_SEC);
        Duration { secs: secs, nanos: nanos }
    }
}

impl Mul<u32> for Duration {
    type Output = Duration;

    fn mul(self, rhs: u32) -> Duration {
        // Multiply nanoseconds as u64, because it cannot overflow that way.
        let total_nanos = self.nanos as u64 * rhs as u64;
        let extra_secs = total_nanos / (NANOS_PER_SEC as u64);
        let nanos = (total_nanos % (NANOS_PER_SEC as u64)) as u32;
        let secs = self.secs.checked_mul(rhs as u64)
                       .and_then(|s| s.checked_add(extra_secs))
                       .expect("overflow when multiplying duration");
        debug_assert!(nanos < NANOS_PER_SEC);
        Duration { secs: secs, nanos: nanos }
    }
}

impl Div<u32> for Duration {
    type Output = Duration;

    fn div(self, rhs: u32) -> Duration {
        let secs = self.secs / (rhs as u64);
        let carry = self.secs - secs * (rhs as u64);
        let extra_nanos = carry * (NANOS_PER_SEC as u64) / (rhs as u64);
        let nanos = self.nanos / rhs + (extra_nanos as u32);
        debug_assert!(nanos < NANOS_PER_SEC);
        Duration { secs: secs, nanos: nanos }
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

        Instant(Duration::new(tp.tv_sec as u64, tp.tv_nsec as u32))
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

        SystemTime(Duration::new(tp.tv_sec as u64, tp.tv_nsec as u32))
    }
}
