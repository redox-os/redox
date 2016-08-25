const SERIAL_BASE: *mut u8 = 0x16000000 as *mut u8;
const SERIAL_FLAG_REGISTER: *const u8 = 0x16000018 as *const u8;
const SERIAL_BUFFER_FULL: u8 =  (1 << 5);

unsafe fn putc (c: u8)
{
    /* Wait until the serial buffer is empty */
    while *SERIAL_FLAG_REGISTER & SERIAL_BUFFER_FULL == SERIAL_BUFFER_FULL {}

    /* Put our character, c, into the serial buffer */
    *SERIAL_BASE = c;
}

unsafe fn puts(string: &str)
{
    for b in string.bytes() {
        putc(b);
    }
}

#[no_mangle]
#[naked]
pub unsafe extern fn kstart() -> ! {
    asm!("mov sp, 0x18000" : : : : "volatile");
    puts("TEST\r\n");
    loop {}
}
