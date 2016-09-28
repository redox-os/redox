extern crate octavo;
extern crate syscall;
extern crate termion;

use octavo::octavo_digest::Digest;
use octavo::octavo_digest::sha3::Sha512;
use std::io::{Read, Write};
use std::process::Command;
use std::{env, io, thread};
use termion::input::TermRead;

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

    env::set_var("COLUMNS", "80");
    env::set_var("LINES", "30");
    env::set_var("TTY", &tty);

    thread::spawn(move || {
        let stdin = io::stdin();
        let mut stdin = stdin.lock();
        let stdout = io::stdout();
        let mut stdout = stdout.lock();

        loop {
            stdout.write_all(b"\x1B[1mredox login:\x1B[0m ").expect("login: failed to write user prompt");
            let _ = stdout.flush();

            let user = (&mut stdin as &mut Read).read_line().expect("login: failed to read user").unwrap_or(String::new());
            if ! user.is_empty() {
                stdout.write_all(b"\x1B[1mpassword:\x1B[0m ").expect("login: failed to write password prompt");
                let _ = stdout.flush();

                if let Some(password) = stdin.read_passwd(&mut stdout).expect("login: failed to read password") {
                    let mut output = vec![0; Sha512::output_bytes()];
                    let mut hash = Sha512::default();
                    hash.update(&password.as_bytes());
                    hash.result(&mut output);

                    println!("");

                    print!("hash: {}: '{}' ", user, password);
                    for b in output.iter() {
                        print!("{:X} ", b);
                    }
                    println!("");

                    let home = "file:home";

                    env::set_current_dir(home).expect("login: failed to cd to home");

                    let mut command = Command::new(&sh);
                    for arg in sh_args.iter() {
                        command.arg(arg);
                    }

                    command.env("USER", &user);
                    command.env("HOME", home);
                    command.env("PATH", "file:bin");

                    match command.spawn() {
                        Ok(mut child) => match child.wait() {
                            Ok(_status) => (), //println!("login: waited for {}: {:?}", sh, status.code()),
                            Err(err) => panic!("login: failed to wait for '{}': {}", sh, err)
                        },
                        Err(err) => panic!("login: failed to execute '{}': {}", sh, err)
                    }

                    stdout.write(b"\x1Bc").expect("login: failed to reset screen");
                    let _ = stdout.flush();
                }
            }
        }
    });
}
