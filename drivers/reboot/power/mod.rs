extern crate io;

use self::io::{Io, Pio};

pub fn reset() {
    let mut port: Pio<u8> = Pio::new(0x64);
    while port.readf(0x02) {}
    port.write(0xFE);
}
