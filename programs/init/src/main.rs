extern crate syscall;

use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Result};
use std::process::Command;

pub fn run(file: &str) -> Result<()> {
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
                        if let Err(err) = run(&new_file) {
                            println!("init: failed to run '{}': {}", new_file, err);
                        }
                    } else {
                        println!("init: failed to run: no argument");
                    },
                    "stdio" => if let Some(stdio) = args.next() {
                        let _ = syscall::close(2);
                        let _ = syscall::close(1);
                        let _ = syscall::close(0);

                        let _ = syscall::open(&stdio, syscall::flag::O_RDWR);
                        let _ = syscall::open(&stdio, syscall::flag::O_RDWR);
                        let _ = syscall::open(&stdio, syscall::flag::O_RDWR);
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
    if let Err(err) = run("initfs:etc/init.rc") {
        println!("init: failed to run initfs:etc/init.rc: {}", err);
    }

    loop {
        let mut status = 0;
        syscall::waitpid(0, &mut status, 0).unwrap();
    }
}
