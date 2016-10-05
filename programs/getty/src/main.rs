#![feature(question_mark)]

extern crate syscall;

use std::io::Write;
use std::process::Command;
use std::{env, io, str, thread};

pub fn main() {
    let mut args = env::args().skip(1);

    let tty = args.next().expect("getty: no tty provided");

    let _ = syscall::close(2);
    let _ = syscall::close(1);
    let _ = syscall::close(0);

    let _ = syscall::open(&tty, syscall::flag::O_RDWR);
    let _ = syscall::open(&tty, syscall::flag::O_RDWR);
    let _ = syscall::open(&tty, syscall::flag::O_RDWR);

    env::set_current_dir("file:").unwrap();

    env::set_var("TTY", &tty);
    {
        let mut path = [0; 4096];
        if let Ok(count) = syscall::fpath(0, &mut path) {
            let path_str = str::from_utf8(&path[..count]).unwrap_or("");
            let reference = path_str.split(':').nth(1).unwrap_or("");
            let mut parts = reference.split('/');
            env::set_var("COLUMNS", parts.next().unwrap_or("80"));
            env::set_var("LINES", parts.next().unwrap_or("30"));
        }
    }

    thread::spawn(move || {
        let stdout = io::stdout();
        let mut stdout = stdout.lock();
        loop {
            stdout.write(b"\x1Bc").unwrap();
            let _ = stdout.flush();
            match Command::new("file:bin/login").spawn() {
                Ok(mut child) => match child.wait() {
                    Ok(_status) => (), //println!("getty: waited for login: {:?}", status.code()),
                    Err(err) => panic!("getty: failed to wait for login: {}", err)
                },
                Err(err) => panic!("getty: failed to execute login: {}", err)
            }
        }
    });
}
