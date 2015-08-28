use core::ops::{Add, Sub, Mul, Div};

use common::string::*;

pub const NANOS_PER_SEC: i32 = 1000000000;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Duration {
    pub secs: i64,
    pub nanos: i32
}

impl Duration {
    pub fn new(mut secs: i64, mut nanos: i32) -> Duration {
        while nanos >= NANOS_PER_SEC {
            secs += 1;
            nanos -= NANOS_PER_SEC;
        }

        while nanos < 0 {
            secs -= 1;
            nanos += NANOS_PER_SEC;
        }

        return Duration {
            secs: secs,
            nanos: nanos
        };
    }

    //TODO: Format decimal
    pub fn to_string(&self) -> String {
        return String::from_num_signed(self.secs as isize);
    }
}

impl Add for Duration {
    type Output = Duration;

    fn add(self, other: Duration) -> Duration {
        return Duration::new(self.secs + other.secs, self.nanos + other.nanos);
    }
}

impl Sub for Duration {
    type Output = Duration;

    fn sub(self, other: Duration) -> Duration {
        return Duration::new(self.secs - other.secs, self.nanos - other.nanos);
    }
}
