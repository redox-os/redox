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

    //TODO: Error handling
    pub fn read(&mut self, block: usize) -> [u8; 512] {
        let mut data: [u8; 512] = [0; 512];
        self.disk.seek(Seek::Start(block * 512));
        self.disk.read(&mut data);
        return data;
    }

    pub fn write(&mut self, block: usize, data: &[u8; 512]) {
        self.disk.seek(Seek::Start(block * 512));
        self.disk.write(data);
    }
}

//TODO: Find a way to remove all the to_string's
pub fn main() {
    console_title(&"ZFS".to_string());

    println!("Opening ZFS Image".to_string());
    let mut zfs = ZFS::new(File::open(&"zfs.img".to_string()));

    while let Option::Some(line) = readln!() {
        let mut args: Vec<String> = Vec::new();
        for arg in line.split(" ".to_string()) {
            args.push(arg);
        }

        if let Option::Some(command) = args.get(0) {
            println!("# ".to_string() + line);

            if *command == "list".to_string() {
                print_color!("List volumes\n".to_string(), Color::new(127, 255, 127));
            } else if *command == "dump".to_string() || *command == "d".to_string() {
                match args.get(1) {
                    Option::Some(arg) => {
                        let block = arg.to_num();
                        print_color!("Dump block:".to_string() + block, Color::new(127, 255, 127));

                        let data = zfs.read(block);
                        for i in 0..data.len() {
                            if i % 32 == 0 {
                                print!("\n".to_string() + String::from_num_radix(i, 16) + ":");
                            }
                            print!(" ".to_string() + String::from_num_radix(data[i] as usize, 16));
                        }
                        print!("\n".to_string());
                    },
                    Option::None => print_color!("No block specified!\n".to_string(), Color::new(255, 127, 127))
                }
            } else {
                print_color!("Commands: list\n".to_string(), Color::new(127, 127, 255));
            }
        }
    }

    println!("Closing ZFS Image".to_string());
    drop(zfs);
}
