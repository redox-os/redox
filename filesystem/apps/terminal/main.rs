extern crate orbital;

use orbital::Color;

use std::fs::File;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::syscall::*;
use std::thread;

use window::ConsoleWindow;

mod window;

macro_rules! readln {
    () => ({
        let mut buffer = String::new();
        match std::io::stdin().read_line(&mut buffer) {
            Ok(_) => Some(buffer),
            Err(_) => None
        }
    });
}

pub fn pipe() -> [usize; 2] {
    let mut fds = [0; 2];
    SysError::demux(unsafe { sys_pipe2(fds.as_mut_ptr(), 0) }).unwrap();
    fds
}

#[no_mangle] pub fn main() {
    let to_shell_fds = pipe();
    let from_shell_fds = pipe();

    unsafe {
        if SysError::demux(sys_clone(0)).unwrap() == 0{
            //Close STDIO
            sys_close(2);
            sys_close(1);
            sys_close(0);

            //Create piped STDIO
            sys_dup(to_shell_fds[0]);
            sys_dup(from_shell_fds[1]);
            sys_dup(from_shell_fds[1]);

            //Close extra pipes
            sys_close(to_shell_fds[0]);
            sys_close(to_shell_fds[1]);
            sys_close(from_shell_fds[0]);
            sys_close(from_shell_fds[1]);

            println!("Test");

            //Execute the shell
            let shell = "file:/apps/shell/main.bin\0";
            sys_execve(shell.as_ptr(), 0 as *const *const u8);
            panic!("Shell not found");
        } else{
            //Close extra pipes
            sys_close(to_shell_fds[0]);
            sys_close(from_shell_fds[1]);
        }
    };

    let window_read = Arc::new(Mutex::new(ConsoleWindow::new(-1, -1, 576, 400, "Terminal")));
    let window_write = window_read.clone();

    thread::spawn(move || {
        let mut to_shell = unsafe { File::from_fd(to_shell_fds[1]).unwrap() };
        loop {
            if let Some(line) = window_read.lock().read() {
                if line.is_empty() {
                    unsafe { sys_yield() };
                } else {
                    to_shell.write(line.as_bytes()).unwrap();
                }
            } else {
                break;
            }
        }
    });

    let mut from_shell = unsafe { File::from_fd(from_shell_fds[0]).unwrap() };
    loop {
        let mut output = String::new();
        if let Ok(_) = from_shell.read_to_string(&mut output) {
            let mut window = window_write.lock();
            window.print(&output, Color::rgb(255, 255, 255));
            window.sync();
        } else {
            break;
        }
    }
}
