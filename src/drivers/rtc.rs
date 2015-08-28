use common::debug::*;
use common::pio::*;
use common::scheduler::*;

unsafe fn rtc_get(reg: u8) -> u8 {
    outb(0x70, reg);
    return inb(0x71);
}

unsafe fn rtc_wait_for_update(){
    while rtc_get(0xA) & 0x80 != 0x80 {}
    while rtc_get(0xA) & 0x80 == 0x80 {}
}

fn cvt_bcd(value: usize) -> usize {
    return (value & 0xF) + ((value/16) * 10);
}

pub unsafe fn rtc_read() -> i64 {
    let reenable = start_no_ints();

    rtc_wait_for_update();

    let mut second = rtc_get(0) as usize;
    let mut minute = rtc_get(2) as usize;
    let mut hour = rtc_get(4) as usize;
    let mut day = rtc_get(7) as usize;
    let mut month = rtc_get(8) as usize;
    let mut year = rtc_get(9) as usize;

    let register_b = rtc_get(0xB);

    end_no_ints(reenable);

    if register_b & 4 != 4 {
        second = cvt_bcd(second);
        minute = cvt_bcd(minute);
        hour = cvt_bcd(hour & 0x7F) | (hour & 0x80);
        day = cvt_bcd(day);
        month = cvt_bcd(month);
        year = cvt_bcd(year);
    }

    if register_b & 2 != 2 || hour & 0x80 == 0x80 {
        hour = ((hour & 0x7F) + 12) % 24;
    }

    //TODO: Century Register
    year += 2000;

    //Unix time from clock
    let mut secs: i64 = (year as i64 - 1970) * 31536000;

    let mut leap_days = (year as i64 - 1972)/4 + 1;
    if year % 4 == 0 {
        if month <= 2 {
            leap_days -= 1;
        }
    }
    secs += leap_days * 86400;

    match month {
        2 => secs += 2678400,
        3 => secs += 5097600,
        4 => secs += 7776000,
        5 => secs += 10368000,
        6 => secs += 13046400,
        7 => secs += 15638400,
        8 => secs += 18316800,
        9 => secs += 20995200,
        10 => secs += 23587200,
        11 => secs += 26265600,
        12 => secs += 28857600,
        _ => ()
    }

    secs += (day as i64 - 1) * 86400;
    secs += hour as i64 * 3600;
    secs += minute as i64 * 60;
    secs += second as i64;

    d("Year ");
    dd(year);
    dl();

    d("Month ");
    dd(month);
    dl();

    d("Day ");
    dd(day);
    dl();

    d("Hour ");
    dd(hour);
    dl();

    d("Minute ");
    dd(minute);
    dl();

    d("Second ");
    dd(second);
    dl();

    d("Unix Time ");
    dd(secs as usize);
    dl();

    return secs;
}
