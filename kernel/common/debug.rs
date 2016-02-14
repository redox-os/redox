use core::str::StrExt;

use syscall::do_sys_debug;

/// Debug to console
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => ({
        $crate::common::debug::d(&format!($($arg)*));
    });
}

/// Debug new line to console
#[macro_export]
macro_rules! debugln {
    ($fmt:expr) => (debug!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (debug!(concat!($fmt, "\n"), $($arg)*));
}

pub fn d(msg: &str) {
    let _ = do_sys_debug(msg.as_ptr(), msg.len());
}

pub fn db(byte: u8) {
    let _ = do_sys_debug(&byte, 1);
}

pub fn dbh(byte: u8) {
    let high = {
        let temp: u8 = byte / 16;
        if temp <= 9 {
            temp + ('0' as u8)
        } else {
            temp - 10 + ('A' as u8)
        }
    };
    db(high);

    let low = {
        let temp = byte % 16;
        if temp <= 9 {
            temp + ('0' as u8)
        } else {
            temp - 10 + ('A' as u8)
        }
    };
    db(low);
}

pub fn dh(num: usize) {
    if num >= 256 {
        dh(num / 256);
    }
    dbh((num % 256) as u8);
}

pub fn dd(num: usize) {
    if num >= 10 {
        dd(num / 10);
    }
    db('0' as u8 + (num % 10) as u8);
}

pub fn ds(num: isize) {
    if num >= 0 {
        dd(num as usize);
    } else {
        dc('-');
        dd((-num) as usize);
    }
}

pub fn dc(character: char) {
    db(character as u8);
}

pub fn dl() {
    dc('\n');
}
