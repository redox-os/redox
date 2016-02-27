use common::time::Duration;

use drivers::io::{Io, Pio};

fn cvt_bcd(value: usize) -> usize {
    (value & 0xF) + ((value / 16) * 10)
}

/// RTC
pub struct Rtc {
    addr: Pio<u8>,
    data: Pio<u8>,
}

impl Rtc {
    /// Create new empty RTC
    pub fn new() -> Self {
        return Rtc {
            addr: Pio::<u8>::new(0x70),
            data: Pio::<u8>::new(0x71),
        };
    }

    /// Read
    unsafe fn read(&mut self, reg: u8) -> u8 {
        self.addr.write(reg);
        return self.data.read();
    }

    /// Wait
    unsafe fn wait(&mut self) {
        while self.read(0xA) & 0x80 != 0x80 {}
        while self.read(0xA) & 0x80 == 0x80 {}
    }

    /// Get time
    pub fn time(&mut self) -> Duration {
        let mut second;
        let mut minute;
        let mut hour;
        let mut day;
        let mut month;
        let mut year;
        let register_b;
        unsafe {
            self.wait();
            second = self.read(0) as usize;
            minute = self.read(2) as usize;
            hour = self.read(4) as usize;
            day = self.read(7) as usize;
            month = self.read(8) as usize;
            year = self.read(9) as usize;
            register_b = self.read(0xB);
        }

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

        // TODO: Century Register
        year += 2000;

        // Unix time from clock
        let mut secs: i64 = (year as i64 - 1970) * 31536000;

        let mut leap_days = (year as i64 - 1972) / 4 + 1;
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
            _ => (),
        }

        secs += (day as i64 - 1) * 86400;
        secs += hour as i64 * 3600;
        secs += minute as i64 * 60;
        secs += second as i64;

        Duration::new(secs, 0)
    }
}
