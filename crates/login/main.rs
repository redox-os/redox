use std::env;
use std::io::{stdin, stdout, Write};
use std::process::Command;

fn main() {
    loop {
        print!("redox login: ");
        stdout().flush().unwrap();

        let mut buffer = String::new();
        stdin().read_line(&mut buffer).unwrap();

        env::set_current_dir("/home/").unwrap();

        let mut child = Command::new("/bin/sh").spawn().unwrap();
        child.wait().unwrap();
    }
}
