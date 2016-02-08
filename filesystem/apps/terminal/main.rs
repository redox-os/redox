extern crate orbclient;

extern crate system;

use orbclient::Color;

use std::fs::File;
use std::io::{Read, Write};
use std::ops::Deref;
use std::sync::Arc;
use std::thread;

use system::error::Error;
use system::syscall::*;

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
    Error::demux(unsafe { sys_pipe2(fds.as_mut_ptr(), 0) }).unwrap();
    fds
}

fn main() {
    let to_shell_fds = pipe();
    let from_shell_fds = pipe();

    unsafe {
        if Error::demux(sys_clone(0)).unwrap() == 0 {
            // Close STDIO
            sys_close(2);
            sys_close(1);
            sys_close(0);

            // Create piped STDIO
            sys_dup(to_shell_fds[0]);
            sys_dup(from_shell_fds[1]);
            sys_dup(from_shell_fds[1]);

            // Close extra pipes
            sys_close(to_shell_fds[0]);
            sys_close(to_shell_fds[1]);
            sys_close(from_shell_fds[0]);
            sys_close(from_shell_fds[1]);

            // Execute the shell
            let shell = "ion\0";
            sys_execve(shell.as_ptr(), 0 as *const *const u8);
            panic!("Shell not found");
        } else {
            // Close extra pipes
            sys_close(to_shell_fds[0]);
            sys_close(from_shell_fds[1]);
        }
    };

    let window = Arc::new(ConsoleWindow::new(-1, -1, 576, 400, "Terminal"));

    let window_weak = Arc::downgrade(&window);
    thread::spawn(move || {
        let mut from_shell = unsafe { File::from_fd(from_shell_fds[0]).unwrap() };
        loop {
            let mut output = String::new();
            if let Ok(_) = from_shell.read_to_string(&mut output) {
                if let Some(window) = window_weak.upgrade() {
                    let window_ptr =
                        (window.deref() as *const Box<ConsoleWindow>) as *mut Box<ConsoleWindow>;
                    unsafe { &mut *window_ptr }.print(&output, Color::rgb(255, 255, 255));
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    });

    {
        let mut to_shell = unsafe { File::from_fd(to_shell_fds[1]).unwrap() };
        let window_ptr = (window.deref() as *const Box<ConsoleWindow>) as *mut Box<ConsoleWindow>;
        while let Some(mut string) = unsafe { &mut *window_ptr }.read() {
            string.push('\n');
            if let Ok(_) = to_shell.write(&string.into_bytes()) {

            } else {
                break;
            }
        }
    }
}
