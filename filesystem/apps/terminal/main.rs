extern crate orbital;

use orbital::Color;

use std::fs::File;
use std::io::{Read, Write};
use std::syscall::*;

use window::ConsoleWindow;

mod window;

pub fn pipe() -> [usize; 2] {
    let mut fds = [0; 2];
    SysError::demux(unsafe { sys_pipe2(fds.as_mut_ptr(), 0) }).unwrap();
    fds
}

#[no_mangle] pub fn main() {
    let to_shell_fds = pipe();
    let from_shell_fds = pipe();

    let (mut to_shell, mut from_shell) = unsafe {
        if sys_clone(0) > 0{
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

            //Execute the shell
            let shell = "file:/apps/shell/main.bin\0";
            sys_execve(shell.as_ptr(), 0 as *const *const u8);
            panic!("Failed to execute shell");
        } else{
            //Close extra pipes
            sys_close(to_shell_fds[0]);
            sys_close(from_shell_fds[1]);

            //Return the good pipes, turned into files
            (File::from_fd(to_shell_fds[1]).unwrap(), File::from_fd(from_shell_fds[0]).unwrap())
        }
    };

    println!("To: {:?} From: {:?}", to_shell.path(), from_shell.path());

    let mut window = ConsoleWindow::new(-1, -1, 576, 400, "Terminal");
    loop {
        {
            let mut output = String::new();
            from_shell.read_to_string(&mut output).unwrap();
            window.print(&output, Color::rgb(255, 255, 255));
        }
        if let Some(line) = window.read() {
            to_shell.write(line.as_bytes()).unwrap();
        } else {
            break;
        }
    }
}
