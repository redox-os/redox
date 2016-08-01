#![deny(warnings)]
#![feature(question_mark)]

use std::env;
use std::fs::File;
use std::io::{stdin, stdout, Result, Read, Write};
use std::process::{Command, ExitStatus};

fn login(user: &str) -> Result<ExitStatus> {
    let shell = "/bin/sh";
    let home = "/home/";
    env::set_current_dir(home)?;

    if let Ok(mut motd) = File::open("/etc/motd") {
        let mut motd_string = String::new();
        if let Ok(_) = motd.read_to_string(&mut motd_string) {
            println!("{}", motd_string);
        }
    }

    Command::new("/bin/sh")
            .env("HOME", home)
            .env("SHELL", shell)
            .env("USER", user)
            .spawn()?.wait()
}

fn main() {
    loop {
        if let Ok(mut issue) = File::open("/etc/issue") {
            let mut issue_string = String::new();
            if let Ok(_) = issue.read_to_string(&mut issue_string) {
                println!("{}", issue_string);
            }
        }

        print!("redox login: ");
        stdout().flush().unwrap();

        let mut user = String::new();
        stdin().read_line(&mut user).unwrap();

        match login(&user) {
            Ok(_exit_status) => (),
            Err(err) => println!("login: failed to login as {}: {}", user, err)
        }
    }
}
