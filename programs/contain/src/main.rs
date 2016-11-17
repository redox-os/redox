extern crate syscall;

use std::os::unix::process::CommandExt;
use std::process::Command;

pub fn main() {
    let names = [
        "file",
        "rand",
        "tcp",
        "udp"
    ];

    let command = "sh";

    let pid = unsafe { syscall::clone(0).unwrap() };
    if pid == 0 {
        let mut name_ptrs = Vec::new();
        for name in names.iter() {
            name_ptrs.push([name.as_ptr() as usize, name.len()]);
        }

        syscall::setns(&name_ptrs).unwrap();

        println!("Container enter: {}", command);

        let err = Command::new(command).exec();

        panic!("contain: failed to launch {}: {}", command, err);
    } else {
        let mut status = 0;
        syscall::waitpid(pid, &mut status, 0).unwrap();

        loop {
            let mut c_status = 0;
            let c_pid = syscall::waitpid(0, &mut c_status, syscall::WNOHANG).unwrap();
            if c_pid == 0 {
                break;
            } else {
                println!("Container zombie {}: {:X}", c_pid, c_status);
            }
        }

        println!("Container exited: {:X}", status);
    }
}
