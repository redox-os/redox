use std::io::{stdin, stdout, Read, Write};
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
                child.wait();
                /*match  {
                    Ok(status) => {
                        if let Some(code) = status.code() {
                            println!("{}: Child exited with exit code: {}", path, code);
                        } else {
                            println!("{}: No child exit code", path);
                        }
                    },
                    Err(err) => println!("{}: Failed to wait: {}", path, err)
                }*/
            },
            Err(err) => println!("{}: Failed to execute: {}", path, err)
        }
    }
}
