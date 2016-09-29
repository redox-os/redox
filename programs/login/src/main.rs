#![feature(question_mark)]

extern crate octavo;
extern crate syscall;
extern crate termion;

use octavo::octavo_digest::Digest;
use octavo::octavo_digest::sha3::Sha512;
use std::fs::File;
use std::io::{Read, Write};
use std::process::Command;
use std::{env, io, str, thread};
use termion::input::TermRead;

pub struct Passwd<'a> {
    user: &'a str,
    hash: &'a str,
    uid: usize,
    gid: usize,
    name: &'a str,
    home: &'a str,
    shell: &'a str
}

impl<'a> Passwd<'a> {
    pub fn parse(line: &'a str) -> Result<Passwd<'a>, ()> {
        let mut parts = line.split(';');

        let user = parts.next().ok_or(())?;
        let hash = parts.next().ok_or(())?;
        let uid = parts.next().ok_or(())?.parse::<usize>().or(Err(()))?;
        let gid = parts.next().ok_or(())?.parse::<usize>().or(Err(()))?;
        let name = parts.next().ok_or(())?;
        let home = parts.next().ok_or(())?;
        let shell = parts.next().ok_or(())?;

        Ok(Passwd {
            user: user,
            hash: hash,
            uid: uid,
            gid: gid,
            name: name,
            home: home,
            shell: shell
        })
    }
}

pub fn main() {
    let mut args = env::args().skip(1);

    let tty = args.next().expect("login: no tty provided");

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
        let stdin = io::stdin();
        let mut stdin = stdin.lock();
        let stdout = io::stdout();
        let mut stdout = stdout.lock();

        loop {
            stdout.write_all(b"\x1B[1mredox login:\x1B[0m ").unwrap();
            let _ = stdout.flush();

            let user = (&mut stdin as &mut Read).read_line().unwrap().unwrap_or(String::new());
            if ! user.is_empty() {
                stdout.write_all(b"\x1B[1mpassword:\x1B[0m ").unwrap();
                let _ = stdout.flush();

                if let Some(password) = stdin.read_passwd(&mut stdout).unwrap() {
                    let password_hash = {
                        let mut output = vec![0; Sha512::output_bytes()];
                        let mut hash = Sha512::default();
                        hash.update(&password.as_bytes());
                        hash.result(&mut output);
                        let mut encoded = String::new();
                        for b in output.iter() {
                            encoded.push_str(&format!("{:X}", b));
                        }
                        encoded
                    };

                    {
                        let mut debug = File::open("debug:").unwrap();
                        write!(debug, "{};{}\n", user, password_hash).unwrap();
                    }

                    let mut passwd_string = String::new();
                    File::open("file:etc/passwd").unwrap().read_to_string(&mut passwd_string).unwrap();

                    let mut passwd_option = None;
                    for line in passwd_string.lines() {
                        if let Ok(passwd) = Passwd::parse(line) {
                            if user == passwd.user && password_hash == passwd.hash {
                                passwd_option = Some(passwd);
                                break;
                            }
                        }
                    }

                    if let Some(passwd) = passwd_option  {
                        stdout.write(b"\n").unwrap();
                        let _ = stdout.flush();

                        let mut command = Command::new(passwd.shell);

                        env::set_current_dir(passwd.home).unwrap();

                        command.env("USER", &user);
                        command.env("HOME", passwd.home);
                        command.env("PATH", "file:bin");

                        match command.spawn() {
                            Ok(mut child) => match child.wait() {
                                Ok(_status) => (), //println!("login: waited for {}: {:?}", sh, status.code()),
                                Err(err) => panic!("login: failed to wait for '{}': {}", passwd.shell, err)
                            },
                            Err(err) => panic!("login: failed to execute '{}': {}", passwd.shell, err)
                        }

                        env::set_current_dir("file:").unwrap();

                        stdout.write(b"\x1Bc").unwrap();
                        let _ = stdout.flush();
                    } else {
                        stdout.write(b"\nLogin failed\n").unwrap();
                        let _ = stdout.flush();
                    }
                }
            }
        }
    });
}
