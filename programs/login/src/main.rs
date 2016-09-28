extern crate liner;
extern crate octavo;
extern crate syscall;

use liner::Context;
use octavo::octavo_digest::Digest;
use octavo::octavo_digest::sha3::Sha512;
use std::process::Command;
use std::{env, thread};

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
        let mut con = Context::new();

        loop {
            let user = con.read_line("\x1B[1mredox login:\x1B[0m ", &mut |_| {}).expect("login: failed to read user");

            if ! user.is_empty() {
                let password = con.read_line("\x1B[1mpassword:\x1B[0m ", &mut |_| {}).expect("login: failed to read user");

                let mut output = vec![0; Sha512::output_bytes()];
                let mut hash = Sha512::default();
                hash.update(&password.as_bytes());
                hash.result(&mut output);

                print!("hash: ");
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
            }
        }
    });
}
