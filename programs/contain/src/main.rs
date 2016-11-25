extern crate syscall;

use std::env;
use std::os::unix::process::CommandExt;
use std::process::Command;

pub fn main() {
    let mut args = env::args().skip(1);

    let root = args.next();

    let cmd = args.next().unwrap_or("sh".to_string());

    let mut names = vec![
        "rand",
        "tcp",
        "udp"
    ];
    
    if root.is_none() {
        names.push("file");
    }

    let mut name_ptrs = Vec::new();
    for name in names.iter() {
        name_ptrs.push([name.as_ptr() as usize, name.len()]);
    }

    let new_ns = syscall::mkns(&name_ptrs).unwrap();

    let pid = unsafe { syscall::clone(0).unwrap() };
    if pid == 0 {
        syscall::setrens(new_ns, new_ns).unwrap();

        println!("Container {}: enter: {}", new_ns, cmd);

        let mut command = Command::new(&cmd);
        for arg in args {
            command.arg(&arg);
        }

        let err = command.exec();

        panic!("contain: failed to launch {}: {}", cmd, err);
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

        println!("Container {}: exit: {:X}", new_ns, status);
    }
}
