use std::io::*;
use std::process::Command;

macro_rules! readln {
    () => ({
        let mut buffer = String::new();
        match std::io::stdin().read_to_string(&mut buffer) {
            Some(_) => Some(buffer),
            None => None
        }
    });
}

#[no_mangle]
pub fn main() {
    loop {
        print!("redox login: ");
        readln!();

        let path = "file:/apps/shell/main.bin";
        if let Some(mut child) = Command::new(path).spawn() {
            if let Some(status) = child.wait() {
                if let Some(code) = status.code() {
                    println!("{}: Child exited with exit code: {}", path, code);
                } else {
                    println!("{}: No child exit code", path);
                }
            } else {
                println!("{}: Failed to wait", path);
            }
        } else {
            println!("{}: Failed to execute", path);
        }
    }
}
