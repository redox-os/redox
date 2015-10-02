//To use this, please install zfs-fuse

use redox::*;

pub struct ZFS {
    disk: File,
}

impl ZFS {
    pub fn new(disk: File) -> ZFS {
        ZFS {
            disk: disk,
        }
    }
}

//TODO: Find a way to remove all the to_string's
pub fn main() {
    console_title(&"ZFS".to_string());

    println!("Opening ZFS Image".to_string());
    let zfs = ZFS::new(File::open(&"zfs.img".to_string()));

    while let Option::Some(line) = readln!() {
        let mut args: Vec<String> = Vec::new();
        for arg in line.split(" ".to_string()) {
            args.push(arg);
        }

        if let Option::Some(command) = args.get(0) {
            print_color!(command.clone() + "\n", Color::new(255, 128, 128));

            if *command == "ls".to_string() {
                println!("List files".to_string());
            }else{
                println!("Command not found".to_string());
            }
        }
    }

    println!("Closing ZFS Image".to_string());
    drop(zfs);
}
