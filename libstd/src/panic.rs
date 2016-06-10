use core::fmt::{self, Write};
use core::result;

use system::syscall::{sys_write, sys_exit};

pub struct DebugStream;

impl Write for DebugStream {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if let Err(_err) = sys_write(2, s.as_bytes()) {
            result::Result::Err(fmt::Error)
        } else {
            result::Result::Ok(())
        }
    }
}

#[lang="panic_fmt"]
#[allow(unused_must_use)]
pub extern "C" fn panic_impl(args: &fmt::Arguments, file: &'static str, line: u32) -> ! {
    let mut stream = DebugStream;
    stream.write_fmt(format_args!("Panic in {}:{}: {}\n", file, line, *args));

    loop {
        let _ = sys_exit(128);
    }
}
