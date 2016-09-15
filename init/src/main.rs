use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process::Command;

pub fn main() {
    let file = File::open("initfs:etc/init.rc").expect("failed to open init.rc");
    let reader = BufReader::new(file);

    for line_result in reader.lines() {
        let line = line_result.expect("failed to read init.rc");
        let line = line.trim();
        if ! line.is_empty() && ! line.starts_with('#') {
            let mut args = line.split(' ');
            if let Some(cmd) = args.next() {
                match cmd {
                    "echo" => {
                        if let Some(arg) = args.next() {
                            print!("{}", arg);
                        }
                        for arg in args {
                            print!(" {}", arg);
                        }
                        print!("\n");
                    },
                    "cd" => if let Some(dir) = args.next() {
                        if let Err(err) = env::set_current_dir(dir) {
                            println!("init: failed to cd to '{}': {}", dir, err);
                        }
                    } else {
                        println!("init: failed to cd: no argument");
                    },
                    _ => {
                        let mut command = Command::new(cmd);
                        for arg in args {
                            command.arg(arg);
                        }

                        match command.spawn() {
                            Ok(mut child) => if let Err(err) = child.wait() {
                                println!("init: failed to wait for '{}': {}", line, err);
                            },
                            Err(err) => println!("init: failed to execute '{}': {}", line, err)
                        }
                    }
                }
            }
        }
    }
}
