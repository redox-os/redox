use core::slice;

use drivers::io::{Io, Pio};

use system::error::Result;

pub fn serial_log(bytes: &[u8]) {
    let serial_status = Pio::<u8>::new(0x3F8 + 5);
    let mut serial_data = Pio::<u8>::new(0x3F8);

    for byte in bytes.iter() {
        while !serial_status.readf(0x20) {}
        serial_data.write(*byte);

        if *byte == 8 {
            while !serial_status.readf(0x20) {}
            serial_data.write(0x20);

            while !serial_status.readf(0x20) {}
            serial_data.write(8);
        }
    }
}

pub fn do_sys_debug(ptr: *const u8, len: usize) -> Result<usize> {
    let bytes = unsafe { slice::from_raw_parts(ptr, len) };

    if unsafe { ::ENV_PTR.is_some() } {
        ::env().console.lock().write(bytes);
    } else {
        serial_log(bytes);
    }

    Ok(len)
}
