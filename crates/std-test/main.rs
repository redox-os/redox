#![feature(asm)]
#![feature(rand)]
#![feature(slice_concat_ext)]

use std::io::{Write, stdin, stdout};
use std::rand;
use std::process;
use std::ptr;
use std::slice::SliceConcatExt;
use std::string::*;
use std::thread;

macro_rules! readln {
    () => ({
        let mut buffer = String::new();
        match stdin().read_line(&mut buffer) {
            Ok(_) => Some(buffer),
            Err(_) => None
        }
    });
}

fn main() {
    println!("Type help for a command list");
    loop {
        print!("# ");
        stdout().flush();

        if let Some(line) = readln!() {
            let args: Vec<String> = line.trim().split(' ').map(|arg| arg.to_string()).collect();

            if let Some(command) = args.get(0) {
                if command == "exit" {
                    process::exit(0);
                } else if command == "panic" {
                    panic!("Test panic")
                } else if command == "ptr_write" {
                    let a_ptr = rand() as *mut u8;
                    unsafe {
                        ptr::write(a_ptr, rand() as u8);
                    }
                } else if command == "reboot" {
                    unsafe {
                        let mut good: u8 = 2;
                        while good & 2 == 2 {
                            asm!("in al, dx" : "={al}"(good) : "{dx}"(0x64) : : "intel", "volatile");
                        }
                        asm!("out dx, al" : : "{dx}"(0x64), "{al}"(0xFE) : : "intel", "volatile");
                    }
                } else if command == "halt" {
                    loop {
                        unsafe {
                            asm!("cli" : : : : "intel", "volatile");
                            asm!("hlt" : : : : "intel", "volatile");
                        }
                    }
                } else if command == "clone" {
                    let parent_message = "Parent Message";
                    let handle = thread::spawn(move || {
                        println!("Child after spawn: {}", parent_message);
                        return "Child message";
                    });
                    println!("Parent after spawn: {}", parent_message);
                    match handle.join() {
                        Some(child_message) => println!("Parent after join: {}", child_message),
                        None => println!("Failed to join"),
                    }
                } else if command == "leak_test" {
                    let mut stack_it: Vec<u8> = Vec::new();
                    loop {
                        stack_it.extend_from_slice(&[0; 4096]);
                    }
                } else if command == "int3" {
                    unsafe {
                        asm!("int 3" : : : : "intel", "volatile");
                    }
                } else {
                    println!("Commands: exit panic ptr_write reboot halt clone leak_test int3");
                }
            }
        } else {
            println!("Failed to read line from stdin");
        }
    }
}
