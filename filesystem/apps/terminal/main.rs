use std::fs::File;
use std::process::Command;
use std::syscall::sys_close;

#[no_mangle]
pub fn main() {
    unsafe {
        sys_close(2);
        sys_close(1);
        sys_close(0);
    }

    let stdin = File::open("terminal:Terminal").unwrap();
    let stdout = stdin.dup().unwrap();
    let stderr = stdout.dup().unwrap();

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

    drop(stderr);
    drop(stdout);
    drop(stdin);
}
