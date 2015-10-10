#![feature(alloc)]
#![feature(core)]

extern crate alloc;
extern crate core;

use alloc::boxed::Box;
use std::{io, fs, rand};
use core::ptr;

macro_rules! readln {
    () => {
        {
            let mut line = String::new();
            match io::stdin().read_line(&mut line) {
                Ok(n) => Some(line.trim().to_string()),
                Err(e) => None
            }
        }
    };
}

fn console_title(title: &str){

}

#[no_mangle]
pub fn main() {
    console_title("Test");

    println!("Type help for a command list");
    while let Some(line) = readln!() {
        let args: Vec<String> = line.split(' ').map(|arg| arg.to_string()).collect();

        if let Some(command) = args.get(0) {
            println!("# {}", line);
            let console_commands = ["panic", "ls", "ptr_write"];

            match &command[..]
            {
                command if command == console_commands[0] => panic!("Test panic"),
                command if command == console_commands[1] => {
                    // TODO: import std::fs functions into libredox
                    //fs::read_dir("/").unwrap().map(|dir| println!("{}", dir));
                }
                command if command == console_commands[2] => {
                    let a_ptr = rand() as *mut u8;
                    // TODO: import Box::{from_raw, to_raw} methods in libredox
                    //let mut a_box = Box::new(rand() as u8);
                    unsafe {
                        ptr::write(a_ptr, rand() as u8);
                        //ptr::write(a_box.to_raw(), rand() as u8);
                    }
                }
                _ => println!("Commands: {}", console_commands.join(" ")),
            }
        }
    }
}
