use std::io::{stdin, stdout, Write};
use std::process::Command;

#[no_mangle] pub fn main() {
    loop {
        print!("redox login: ");
        stdout().flush();

        let mut buffer = String::new();
        stdin().read_line(&mut buffer);

        let path = "/apps/shell/main.bin";
        match Command::new(path).spawn() {
            Ok(mut child) => {
                if let Err(err) = child.wait() {
                    println!("{}: Failed to wait: {}", path, err)
                }
            },
            Err(err) => println!("{}: Failed to execute: {}", path, err)
        }
    }
}
