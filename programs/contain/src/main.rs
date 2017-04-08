extern crate syscall;

use syscall::scheme::Scheme;

use std::{env, fs,thread};
use std::io::{stderr, Write};
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{self, Command};

use self::chroot::ChrootScheme;

mod chroot;

fn usage() -> ! {
    write!(stderr(), "contain root cmd args..\n").unwrap();
    process::exit(1);
}

fn enter(root: &Path, cmd: &str, args: &[String]) {
    let names = [
        "pty",
        "rand",
        "tcp",
        "udp"
    ];

    let mut name_ptrs = Vec::new();
    for name in names.iter() {
        name_ptrs.push([name.as_ptr() as usize, name.len()]);
    }

    let new_ns = syscall::mkns(&name_ptrs).unwrap();

    let root_canon = fs::canonicalize(root).unwrap();
    let root_thread = thread::spawn(move || {
        syscall::setrens(-1isize as usize, new_ns).unwrap();
        let scheme_fd = syscall::open(":file", syscall::O_CREAT | syscall::O_RDWR | syscall::O_CLOEXEC).unwrap();
        syscall::setrens(-1isize as usize, syscall::getns().unwrap()).unwrap();

        let chroot_scheme = ChrootScheme::new(root_canon);
        loop {
            let mut packet = syscall::Packet::default();
            if syscall::read(scheme_fd, &mut packet).unwrap() == 0 {
                break;
            }
            chroot_scheme.handle(&mut packet);
            syscall::write(scheme_fd, &packet).unwrap();
        }

        let _ = syscall::close(scheme_fd);
    });

    let pid = unsafe { syscall::clone(0).unwrap() };
    if pid == 0 {
        syscall::setrens(new_ns, new_ns).unwrap();

        println!("Container {}: enter: {}", new_ns, cmd);

        let mut command = Command::new(&cmd);
        for arg in args {
            command.arg(&arg);
        }
        command.current_dir("/");

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

pub fn main() {
    let mut args = env::args().skip(1);

    if let Some(root) = args.next() {
        let cmd = args.next().unwrap_or(env::var("SHELL").unwrap_or("sh".to_string()));
        let args: Vec<String> = args.collect();
        enter(Path::new(&root), &cmd, &args);
    } else {
        usage();
    }
}
