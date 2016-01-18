use std::fs::File;
use std::io::{Read, Write};
use std::process::Command;

fn main() {
    let mut file = File::open("/apps/init/cmds").unwrap();

    let mut string = String::new();
    file.read_to_string(&mut string).unwrap();

    let mut children = Vec::new();
    for line in string.lines() {
        match Command::new(line).spawn() {
            Ok(child) => children.push(child),
            Err(err) => println!("{}: Failed to execute: {}", line, err),
        }
    }

    for mut child in children.iter_mut() {
        if let Err(err) = child.wait() {
            println!("Failed to wait: {}", err)
        }
    }
}
