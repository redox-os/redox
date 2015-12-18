extern crate orbital;

use orbital::Color;

use std::syscall::{SysError, sys_pipe2};

use window::ConsoleWindow;

mod window;

#[no_mangle] pub fn main() {
    let mut window = ConsoleWindow::new(-1, -1, 576, 400, "Terminal");

    let mut output_pipe = [0; 2];
    SysError::demux(unsafe { sys_pipe2(output_pipe.as_mut_ptr(), 0) }).unwrap();

    let mut input_pipe = [0; 2];
    SysError::demux(unsafe { sys_pipe2(input_pipe.as_mut_ptr(), 0) }).unwrap();

    loop {
        window.print("# ", Color::rgb(255, 255, 255));
        if let Some(line) = window.read() {
            window.print(&format!("{}\n", line), Color::rgb(224, 224, 224));
        } else {
            break;
        }
    }
}
