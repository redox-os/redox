use core::str::StrExt;

use syscall::call::sys_debug;

/// Set debug level
pub fn db(byte: u8) {
    unsafe {
        sys_debug(byte);
    }
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

pub fn d(text: &str) {
    for character in text.chars() {
        dc(character);
    }
}
