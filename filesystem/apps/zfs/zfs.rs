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

    println!("Type open zfs.img to open the image file".to_string());
    println!("This may take up to 30 seconds".to_string());

    let mut zfs_option: Option<ZFS> = Option::None;

    while let Option::Some(line) = readln!() {
        let mut args: Vec<String> = Vec::new();
        for arg in line.split(" ".to_string()) {
            args.push(arg);
        }

        if let Option::Some(command) = args.get(0) {
            println!("# ".to_string() + line);

            let mut close = false;
            match zfs_option {
                Option::Some(ref mut zfs) => {
                    if *command == "list".to_string() {
                        print_color!("List volumes\n".to_string(), Color::new(127, 255, 127));
                    } else if *command == "dump".to_string() {
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
                    }else if *command == "close".to_string() {
                        print_color!("Closing\n".to_string(), Color::new(255, 127, 127));
                        close = true;
                    } else {
                        print_color!("Commands: list dump close\n".to_string(), Color::new(127, 127, 255));
                    }
                },
                Option::None => {
                    if *command == "open".to_string() {
                        match args.get(1) {
                            Option::Some(arg) => {
                                print_color!("Open: ".to_string() + arg.clone() + "\n", Color::new(127, 255, 127));
                                zfs_option = Option::Some(ZFS::new(File::open(arg)));
                            },
                            Option::None => print_color!("No file specified!\n".to_string(), Color::new(255, 127, 127))
                        }
                    }else{
                        print_color!("Commands: open\n".to_string(), Color::new(127, 127, 255));
                    }
                }
            }
            if close {
                zfs_option = Option::None;
            }
        }
    }
}
