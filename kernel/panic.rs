use core::{fmt, result};

use common::debug;

use syscall::handle::do_sys_exit;

struct DebugStream;

impl fmt::Write for DebugStream {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        debug::d(s);

        result::Result::Ok(())
    }
}

#[lang="panic_fmt"]
pub extern fn panic_fmt(args: fmt::Arguments, file: &'static str, line: u32) -> ! {
    debug::d(file);
    debug::d(":");
    debug::dd(line as usize);
    debug::d(": ");
    fmt::write(&mut DebugStream, args);
    debug::dl();

    unsafe {
        do_sys_exit(-1);
        loop {
            asm!("sti");
            asm!("hlt");
        }
    }
}
