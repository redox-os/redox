#![deny(warnings)]

extern crate syscall;

use std::env;
use std::fs::{File, OpenOptions, read_dir};
use std::io::{BufRead, BufReader, Error, Result};
use std::os::unix::io::AsRawFd;
use std::path::Path;
use std::process::Command;

fn switch_stdio(stdio: &str) -> Result<()> {
    let stdin = OpenOptions::new().read(true).open(stdio)?;
    let stdout = OpenOptions::new().write(true).open(stdio)?;
    let stderr = OpenOptions::new().write(true).open(stdio)?;

    syscall::dup2(stdin.as_raw_fd(), 0, &[]).map_err(|err| Error::from_raw_os_error(err.errno))?;
    syscall::dup2(stdout.as_raw_fd(), 0, &[]).map_err(|err| Error::from_raw_os_error(err.errno))?;
    syscall::dup2(stderr.as_raw_fd(), 0, &[]).map_err(|err| Error::from_raw_os_error(err.errno))?;

    Ok(())
}

pub fn run(file: &Path) -> Result<()> {
    let file = File::open(file)?;
    let reader = BufReader::new(file);

    for line_result in reader.lines() {
        let line = line_result?;
        let line = line.trim();
        if ! line.is_empty() && ! line.starts_with('#') {
            let mut args = line.split(' ');
            if let Some(cmd) = args.next() {
                match cmd {
                    "cd" => if let Some(dir) = args.next() {
                        if let Err(err) = env::set_current_dir(dir) {
                            println!("init: failed to cd to '{}': {}", dir, err);
                        }
                    } else {
                        println!("init: failed to cd: no argument");
                    },
                    "echo" => {
                        if let Some(arg) = args.next() {
                            print!("{}", arg);
                        }
                        for arg in args {
                            print!(" {}", arg);
                        }
                        print!("\n");
                    },
                    "export" => if let Some(var) = args.next() {
                        let mut value = String::new();
                        if let Some(arg) = args.next() {
                            value.push_str(&arg);
                        }
                        for arg in args {
                            value.push(' ');
                            value.push_str(&arg);
                        }
                        env::set_var(var, value);
                    } else {
                        println!("init: failed to export: no argument");
                    },
                    "run" => if let Some(new_file) = args.next() {
                        if let Err(err) = run(&Path::new(new_file)) {
                            println!("init: failed to run '{}': {}", new_file, err);
                        }
                    } else {
                        println!("init: failed to run: no argument");
                    },
                    "run.d" => if let Some(new_dir) = args.next() {
                        match read_dir(new_dir) {
                            Ok(list) => {
                                let mut entries = vec![];
                                for entry_res in list {
                                    match entry_res {
                                        Ok(entry) => {
                                            entries.push(entry.path());
                                        },
                                        Err(err) => {
                                            println!("init: failed to run.d: '{}': {}", new_dir, err);
                                        }
                                    }
                                }

                                entries.sort();

                                for entry in entries {
                                    if let Err(err) = run(&entry) {
                                        println!("init: failed to run '{}': {}", entry.display(), err);
                                    }
                                }
                            },
                            Err(err) => {
                                println!("init: failed to run.d: '{}': {}", new_dir, err);
                            }
                        }
                    } else {
                        println!("init: failed to run.d: no argument");
                    },
                    "stdio" => if let Some(stdio) = args.next() {
                        if let Err(err) = switch_stdio(&stdio) {
                            println!("init: failed to switch stdio to '{}': {}", stdio, err);
                        }
                    } else {
                        println!("init: failed to set stdio: no argument");
                    },
                    _ => {
                        let mut command = Command::new(cmd);
                        for arg in args {
                            command.arg(arg);
                        }

                        match command.spawn() {
                            Ok(mut child) => match child.wait() {
                                Ok(_status) => (), //println!("init: waited for {}: {:?}", line, status.code()),
                                Err(err) => println!("init: failed to wait for '{}': {}", line, err)
                            },
                            Err(err) => println!("init: failed to execute '{}': {}", line, err)
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

pub fn main() {
    if let Err(err) = run(&Path::new("initfs:etc/init.rc")) {
        println!("init: failed to run initfs:etc/init.rc: {}", err);
    }

    loop {
        let mut status = 0;
        syscall::waitpid(0, &mut status, 0).unwrap();
    }
}
