#![feature(asm)]

use std::Box;
use std::{io, rand};
use std::ptr;
use std::syscall::sys_fork;

macro_rules! readln {
    () => {
        {
            let mut line = String::new();
            match io::stdin().read_line(&mut line) {
                Ok(_) => Some(line.trim().to_string()),
                Err(_) => None
            }
        }
    };
}

fn console_title(title: &str) {

}

#[no_mangle]
pub fn main() {
    console_title("Test");

    println!("Type help for a command list");
    while let Some(line) = readln!() {
        let args: Vec<String> = line.split(' ').map(|arg| arg.to_string()).collect();

        if let Some(a_command) = args.get(0) {
            println!("# {}", line);
            let console_commands = ["panic",
                                    "ls",
                                    "ptr_write",
                                    "box_write",
                                    "reboot",
                                    "shutdown",
                                    "fork",
                                    "leak_test"];

            match &a_command[..] {
                command if command == console_commands[0] =>
                    panic!("Test panic"),
                command if command == console_commands[1] => {
                    // TODO: import std::fs functions into libredox
                    //fs::read_dir("/").unwrap().map(|dir| println!("{}", dir));
                }
                command if command == console_commands[2] => {
                    let a_ptr = rand() as *mut u8;
                    unsafe { ptr::write(a_ptr, rand() as u8); }
                }
                command if command == console_commands[3] => {
                    let mut a_box = Box::new(rand() as u8);
                    unsafe { ptr::write(Box::into_raw(a_box), rand() as u8); }
                }
                command if command == console_commands[4] => {
                    unsafe {
                        let mut good: u8 = 2;
                        while good & 2 == 2 {
                            asm!("in al, dx" : "={al}"(good) : "{dx}"(0x64) : : "intel", "volatile");
                        }
                        asm!("out dx, al" : : "{dx}"(0x64), "{al}"(0xFE) : : "intel", "volatile");
                        loop {
                            asm!("cli" : : : : "intel", "volatile");
                            asm!("hlt" : : : : "intel", "volatile");
                        }
                    }
                }
                command if command == console_commands[5] => {
                    unsafe {
                        loop {
                            asm!("cli" : : : : "intel", "volatile");
                            asm!("hlt" : : : : "intel", "volatile");
                        }
                    }
                }
                command if command == console_commands[6] => {
                    unsafe {
                        if sys_fork() == 0 {
                            println!("Parent from fork");
                        } else {
                            println!("Child from fork");
                        }
                    }
                }
                command if command == console_commands[7] => {
                    let mut stack_it: Vec<Box<u8>> = Vec::new();
                    loop {
                        stack_it.push(Box::new(rand() as u8))
                    }
                }
                _ => println!("Commands: {}", console_commands.join(" ")),
            }
        }
    }
}
