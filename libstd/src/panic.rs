use core::fmt::{self, Write};
use core::result;

use system::syscall::{sys_debug, sys_exit};

pub struct DebugStream;

impl Write for DebugStream {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        unsafe {
            sys_debug(s.as_ptr(), s.len());
        }

        result::Result::Ok(())
    }
}

#[lang="panic_fmt"]
#[allow(unused_must_use)]
pub extern "C" fn panic_impl(args: fmt::Arguments, file: &'static str, line: u32) -> ! {
    let mut stream = DebugStream;
    fmt::write(&mut stream, args);
    fmt::write(&mut stream, format_args!(" in {}:{}\n", file, line));

    unsafe {
        sys_exit(-1);
        loop {
            asm!("sti");
            asm!("hlt");
        }
    }
}
