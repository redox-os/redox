#![feature(question_mark)]

extern crate syscall;

use std::env;
use std::fs::File;
use std::io::Read;
use std::os::unix::process::CommandExt;
use std::process::{self, Command};

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

pub struct Group<'a> {
    group: &'a str,
    gid: u32,
    users: &'a str,
}

impl<'a> Group<'a> {
    pub fn parse(line: &'a str) -> Result<Group<'a>, ()> {
        let mut parts = line.split(';');

        let group = parts.next().ok_or(())?;
        let gid = parts.next().ok_or(())?.parse::<u32>().or(Err(()))?;
        let users = parts.next().ok_or(())?;

        Ok(Group {
            group: group,
            gid: gid,
            users: users
        })
    }
}

pub fn main() {
    let mut args = env::args().skip(1);
    let cmd = args.next().expect("sudo: no command provided");

    let uid = syscall::getuid().unwrap() as u32;

    if uid != 0 {
        let mut passwd_string = String::new();
        File::open("file:etc/passwd").unwrap().read_to_string(&mut passwd_string).unwrap();

        let mut passwd_option = None;
        for line in passwd_string.lines() {
            if let Ok(passwd) = Passwd::parse(line) {
                if uid == passwd.uid {
                    passwd_option = Some(passwd);
                    break;
                }
            }
        }

        let passwd = passwd_option.expect("sudo: user not found in passwd");

        let mut group_string = String::new();
        File::open("file:etc/group").unwrap().read_to_string(&mut group_string).unwrap();

        let mut group_option = None;
        for line in group_string.lines() {
            if let Ok(group) = Group::parse(line) {
                if group.group == "sudo" && group.users.split(',').any(|name| name == passwd.user) {
                    group_option = Some(group);
                    break;
                }
            }
        }

        if group_option.is_none() {
            panic!("sudo: '{}' not in sudo group", passwd.user);
        }
    }

    let mut command = Command::new(&cmd);
    for arg in args {
        command.arg(&arg);
    }

    command.uid(0);
    command.gid(0);
    command.env("USER", "root");

    match command.spawn() {
        Ok(mut child) => match child.wait() {
            Ok(status) => process::exit(status.code().unwrap_or(0)),
            Err(err) => panic!("sudo: failed to wait for {}: {}", cmd, err)
        },
        Err(err) => panic!("sudo: failed to execute {}: {}", cmd, err)
    }
}
