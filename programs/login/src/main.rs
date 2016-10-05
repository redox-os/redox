#![feature(question_mark)]

extern crate octavo;
extern crate termion;

use octavo::octavo_digest::Digest;
use octavo::octavo_digest::sha3::Sha512;
use std::fs::File;
use std::io::{Read, Write};
use std::process::{Command, CommandExt};
use std::{io, str};
use termion::input::TermRead;

pub struct Passwd<'a> {
    user: &'a str,
    hash: &'a str,
    uid: u32,
    gid: u32,
    name: &'a str,
    home: &'a str,
    shell: &'a str
}

impl<'a> Passwd<'a> {
    pub fn parse(line: &'a str) -> Result<Passwd<'a>, ()> {
        let mut parts = line.split(';');

        let user = parts.next().ok_or(())?;
        let hash = parts.next().ok_or(())?;
        let uid = parts.next().ok_or(())?.parse::<u32>().or(Err(()))?;
        let gid = parts.next().ok_or(())?.parse::<u32>().or(Err(()))?;
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
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    if let Ok(mut issue) = File::open("/etc/issue") {
        io::copy(&mut issue, &mut stdout).unwrap();
        let _ = stdout.flush();
    }

    loop {
        stdout.write_all(b"\x1B[1mredox login:\x1B[0m ").unwrap();
        let _ = stdout.flush();

        let user = (&mut stdin as &mut Read).read_line().unwrap().unwrap_or(String::new());
        if ! user.is_empty() {
            let mut passwd_string = String::new();
            File::open("file:etc/passwd").unwrap().read_to_string(&mut passwd_string).unwrap();

            let mut passwd_option = None;
            for line in passwd_string.lines() {
                if let Ok(passwd) = Passwd::parse(line) {
                    if user == passwd.user && "" == passwd.hash {
                        passwd_option = Some(passwd);
                        break;
                    }
                }
            }

            if passwd_option.is_none() {
                stdout.write_all(b"\x1B[1mpassword:\x1B[0m ").unwrap();
                let _ = stdout.flush();

                if let Some(password) = stdin.read_passwd(&mut stdout).unwrap() {
                    stdout.write(b"\n").unwrap();
                    let _ = stdout.flush();

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

                    if let Ok(mut debug) = File::open("debug:") {
                        let _  = write!(debug, "{};{}\n", user, password_hash);
                    }

                    for line in passwd_string.lines() {
                        if let Ok(passwd) = Passwd::parse(line) {
                            if user == passwd.user && password_hash == passwd.hash {
                                passwd_option = Some(passwd);
                                break;
                            }
                        }
                    }
                }
            }

            if let Some(passwd) = passwd_option  {
                if let Ok(mut motd) = File::open("/etc/motd") {
                    io::copy(&mut motd, &mut stdout).unwrap();
                    let _ = stdout.flush();
                }

                let mut command = Command::new(passwd.shell);

                command.uid(passwd.uid);
                command.gid(passwd.gid);

                command.current_dir(passwd.home);

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

                break;
            } else {
                stdout.write(b"\nLogin failed\n").unwrap();
                let _ = stdout.flush();
            }
        } else {
            stdout.write(b"\n").unwrap();
            let _ = stdout.flush();
        }
    }
}
