extern crate syscall;

use std::env;
use std::process::Command;
use std::thread;

pub fn main() {
    let mut args = env::args().skip(1);

    let tty = args.next().expect("login: no tty provided");
    let sh = args.next().expect("login: no sh provided");
    let sh_args: Vec<String> = args.collect();

    let _ = syscall::close(2);
    let _ = syscall::close(1);
    let _ = syscall::close(0);

    let _ = syscall::open(&tty, syscall::flag::O_RDWR);
    let _ = syscall::open(&tty, syscall::flag::O_RDWR);
    let _ = syscall::open(&tty, syscall::flag::O_RDWR);

    thread::spawn(move || {
        loop {
            let mut command = Command::new(&sh);
            for arg in sh_args.iter() {
                command.arg(arg);
            }

            command.env("HOME", "initfs:");
            command.env("PWD", "initfs:bin");
            command.env("PATH", "initfs:bin");
            command.env("COLUMNS", "80");
            command.env("LINES", "30");
            command.env("TTY", &tty);

            match command.spawn() {
                Ok(mut child) => match child.wait() {
                    Ok(_status) => (), //println!("login: waited for {}: {:?}", sh, status.code()),
                    Err(err) => panic!("login: failed to wait for '{}': {}", sh, err)
                },
                Err(err) => panic!("login: failed to execute '{}': {}", sh, err)
            }
        }
    });
}
