use redox::Box;
use redox::fs;
use redox::io::Read;
use redox::rand;
use redox::ptr;
use redox::slice::SliceConcatExt;
use redox::string::*;
use redox::syscall::sys_exit;
use redox::thread;
use redox::Vec;

pub fn main() {
    println!("Type help for a command list");
    loop {
        print!("# ");
        if let Some(line) = readln!() {
            let args: Vec<String> = line.trim().split(' ').map(|arg| arg.to_string()).collect();

            if let Some(a_command) = args.get(0) {
                let console_commands = ["exit",
                                        "panic",
                                        "ls",
                                        "ptr_write",
                                        "box_write",
                                        "reboot",
                                        "shutdown",
                                        "clone",
                                        "leak_test",
                                        "test_hm",
                                        "int3"];

                match &a_command[..] {
                    command if command == console_commands[0] =>
                        unsafe { sys_exit(0); },
                    command if command == console_commands[1] => panic!("Test panic"),
                    command if command == console_commands[2] => {
                        for entry in fs::read_dir("file:///").unwrap() {
                            println!("{}", entry.path());
                        }
                    }
                    command if command == console_commands[3] => {
                        let a_ptr = rand() as *mut u8;
                        unsafe {
                            ptr::write(a_ptr, rand() as u8);
                        }
                    }
                    command if command == console_commands[4] => {
                        let a_box = Box::new(rand() as u8);
                        unsafe {
                            ptr::write(Box::into_raw(a_box), rand() as u8);
                        }
                    }
                    command if command == console_commands[5] => {
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
                    command if command == console_commands[6] => {
                        unsafe {
                            loop {
                                asm!("cli" : : : : "intel", "volatile");
                                asm!("hlt" : : : : "intel", "volatile");
                            }
                        }
                    }
                    command if command == console_commands[7] => {
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
                    }
                    command if command == console_commands[8] => {
                        let mut stack_it: Vec<Box<u8>> = Vec::new();
                        loop {
                            stack_it.push(Box::new(rand() as u8))
                        }
                    }
                    command if command == console_commands[9] => {
                        ::redox::hashmap::test();
                    }
                    command if command == console_commands[10] => {
                        unsafe {
                            asm!("int 3" : : : : "intel", "volatile");
                        }
                    }
                    _ => println!("Commands: {}", console_commands.join(" ")),
                }
            }
        } else {
            println!("Failed to read line from stdin");
        }
    }
}
