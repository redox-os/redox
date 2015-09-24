use core::cmp::*;
use core::ops::*;

use common::string::*;

use syscall::call::sys_time;
use syscall::call::sys_yield;

pub const NANOS_PER_MICRO: i32 = 1000;
pub const NANOS_PER_MILLI: i32 = 1000000;
pub const NANOS_PER_SEC: i32 = 1000000000;

#[derive(Copy, Clone)]
pub struct Duration {
    pub secs: i64,
    pub nanos: i32
}

impl Duration {
    pub fn new(mut secs: i64, mut nanos: i32) -> Duration {
        while nanos >= NANOS_PER_SEC || (nanos > 0 && secs < 0) {
            secs += 1;
            nanos -= NANOS_PER_SEC;
        }

        while nanos < 0 && secs > 0 {
            secs -= 1;
            nanos += NANOS_PER_SEC;
        }

        return Duration {
            secs: secs,
            nanos: nanos
        };
    }

    pub fn monotonic() -> Duration {
        let mut ret = Duration::new(0, 0);
        unsafe{
            sys_time(&mut ret, false);
        }
        return ret;
    }

    pub fn realtime() -> Duration {
        let mut ret = Duration::new(0, 0);
        unsafe{
            sys_time(&mut ret, true);
        }
        return ret;
    }

    pub fn sleep(&self){
        let start_time = Duration::monotonic();
        loop {
            let elapsed = Duration::monotonic() - start_time;
            if elapsed > *self {
                break;
            }else{
                sys_yield();
            }
        }
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

impl PartialEq for Duration {
    fn eq(&self, other: &Duration) -> bool {
        let dif = *self - *other;
        return dif.secs == 0 && dif.nanos == 0;
    }
}

impl PartialOrd for Duration {
    fn partial_cmp(&self, other: &Duration) -> Option<Ordering> {
        let dif = *self - *other;
        if dif.secs > 0 {
            return Option::Some(Ordering::Greater);
        }else if dif.secs < 0 {
            return Option::Some(Ordering::Less);
        }else if dif.nanos > 0 {
            return Option::Some(Ordering::Greater);
        }else if dif.nanos < 0 {
            return Option::Some(Ordering::Less);
        }else{
            return Option::Some(Ordering::Equal);
        }
    }
}
