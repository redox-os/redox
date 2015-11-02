use redox::Box;
use redox::console::*;
use redox::fs;
use redox::rand;
use redox::ptr;
use redox::slice::SliceConcatExt;
use redox::string::*;
use redox::Vec;

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
                                    "leak_test"];

            match &a_command[..] {
                command if command == console_commands[0] =>
                    panic!("Test panic"),
                command if command == console_commands[1] => {
                    for entry in fs::read_dir("file:///").unwrap() {
                        println!("{}", entry.path());
                    }
                }
                command if command == console_commands[2] => {
                    let a_ptr = rand() as *mut u8;
                    unsafe { ptr::write(a_ptr, rand() as u8); }
                }
                command if command == console_commands[3] => {
                    let a_box = Box::new(rand() as u8);
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
