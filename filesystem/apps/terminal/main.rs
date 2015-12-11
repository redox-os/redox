use std::fs::File;
use std::process::Command;
use std::syscall::sys_close;

#[no_mangle] pub fn main() {
    unsafe {
        sys_close(2);
        sys_close(1);
        sys_close(0);
    }

    let stdin = File::open("terminal:Terminal").unwrap();
    let stdout = stdin.dup().unwrap();
    let stderr = stdout.dup().unwrap();

    let path = "file:/apps/shell/main.bin";
    match Command::new(path).spawn() {
        Ok(mut child) => {
            match child.wait() {
                Ok(status) => {
                    if let Some(code) = status.code() {
                        println!("{}: Child exited with exit code: {}", path, code);
                    } else {
                        println!("{}: No child exit code", path);
                    }
                },
                Err(err) => println!("{}: Failed to wait: {}", path, err)
            }
        },
        Err(err) => println!("{}: Failed to execute: {}", path, err)
    }

    drop(stderr);
    drop(stdout);
    drop(stdin);
}
