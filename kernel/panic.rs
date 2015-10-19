use core::fmt;
use core::result;

use common::debug::*;

use syscall::handle::do_sys_exit;

struct DebugStream;

impl fmt::Write for DebugStream {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        d(s);

        result::Result::Ok(())
    }
}

#[lang="panic_fmt"]
pub extern fn panic_fmt(args: fmt::Arguments, file: &'static str, line: u32) -> ! {
    d(file);
    d(":");
    dd(line as usize);
    d(": ");
    fmt::write(&mut DebugStream, args);
    dl();

    unsafe {
        do_sys_exit(-1);
        loop {
            asm!("sti");
            asm!("hlt");
        }
    }
}
