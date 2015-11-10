use core::fmt;
use panic::panic_impl;

pub fn begin_unwind(string: &'static str, file_line: &(&'static str, u32)) -> ! {
    let &(file, line) = file_line;
    panic_impl(format_args!("{}", string), file, line)
}

pub fn begin_unwind_fmt(fmt: fmt::Arguments, file_line: &(&'static str, u32)) -> ! {
    let &(file, line) = file_line;
    panic_impl(fmt, file, line)
}
