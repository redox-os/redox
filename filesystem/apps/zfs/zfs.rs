//To use this, please install zfs-fuse
use redox::*;

use core::ptr;

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
    pub fn read(&mut self, start: usize, length: usize) -> Vec<u8> {
        let mut ret: Vec<u8> = Vec::new();

        for sector in start..start + length {
            //TODO: Check error
            self.disk.seek(Seek::Start(sector * 512));

            let mut data: [u8; 512] = [0; 512];
            self.disk.read(&mut data);

            for i in 0..512{
                ret.push(data[i]);
            }
        }

        return ret;
    }

    pub fn write(&mut self, block: usize, data: &[u8; 512]) {
        self.disk.seek(Seek::Start(block * 512));
        self.disk.write(data);
    }
}

#[repr(packed)]
pub struct VdevLabel {
    pub blank: [u8; 8 * 1024],
    pub boot_header: [u8; 8 * 1024],
    pub nv_pairs: [u8; 112 * 1024],
    pub uberblocks: [Uberblock; 128]
}

#[derive(Copy, Clone)]
#[repr(packed)]
pub struct Uberblock {
    pub magic: u64,
    pub version: u64,
    pub txg: u64,
    pub guid_sum: u64,
    pub timestamp: u64,
    pub rootbp: BlockPtr
}

impl Uberblock {
    pub fn magic_little() -> u64 {
        return 0x0cb1ba00;
    }

    pub fn magic_big() -> u64 {
        return 0x00bab10c;
    }

    pub fn from(data: &Vec<u8>) -> Option<Uberblock> {
        if data.len() >= 1024 {
            let uberblock = unsafe { ptr::read(data.data as *const Uberblock) };
            if uberblock.magic == Uberblock::magic_little() {
                println!("Little Magic".to_string());
                return Option::Some(uberblock);
            }else if uberblock.magic == Uberblock::magic_big() {
                println!("Big Magic".to_string());
                return Option::Some(uberblock);
            }else if uberblock.magic > 0 {
                println!("Unknown Magic: ".to_string() + uberblock.magic as usize);
            }
        }

        Option::None
    }
}

#[derive(Copy, Clone)]
#[repr(packed)]
pub struct DVAddr {
    pub vdev: u64,
    pub offset: u64,
}

impl DVAddr {
    /// Sector address is the offset plus two vdev labels and one boot block (4 MB, or 8192 sectors)
    pub fn sector(&self) -> u64 {
        self.offset + 0x2000
    }
}

#[derive(Copy, Clone)]
#[repr(packed)]
pub struct BlockPtr {
    pub dvas: [DVAddr; 3],
    pub flags_size: u64,
    pub padding: [u64; 3],
    pub birth_txg: u64,
    pub fill_count: u64,
    pub checksum: [u64; 4],
}

#[derive(Copy, Clone)]
#[repr(packed)]
pub struct Gang {
    pub bps: [BlockPtr; 3],
    pub padding: [u64; 14],
    pub magic: u64,
    pub checksum: u64
}

impl Gang {
    pub fn magic() -> u64 {
        return 0x117a0cb17ada1002;
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
                    if *command == "uber".to_string() {
                        //128 KB of ubers after 128 KB of other stuff
                        let mut newest_uberblock: Option<Uberblock> = Option::None;
                        for i in 0..128 {
                            match Uberblock::from(&zfs.read(256 + i * 2, 2)) {
                                Option::Some(uberblock) => {
                                    let mut newest = false;
                                    match newest_uberblock {
                                        Option::Some(previous) => {
                                            if uberblock.txg > previous.txg {
                                                newest = true;
                                            }
                                        },
                                        Option::None => newest = true
                                    }

                                    if newest {
                                        newest_uberblock = Option::Some(uberblock);
                                    }
                                },
                                Option::None => () //Invalid uberblock
                            }
                        }

                        match newest_uberblock {
                            Option::Some(uberblock) => {
                                print_color!("Newest Uberblock:\n".to_string(), Color::new(127, 255, 127));
                                //TODO: Do not use as usize
                                println!("Magic: ".to_string() + String::from_num_radix(uberblock.magic as usize, 16));
                                println!("Version: ".to_string() + uberblock.version as usize);
                                println!("TXG: ".to_string() + uberblock.txg as usize);
                                println!("Timestamp: ".to_string() + uberblock.timestamp as usize);
                                println!("MOS: ".to_string() + uberblock.rootbp.dvas[0].sector() as usize);
                            },
                            Option::None => print_color!("No valid uberblock found!\n".to_string(), Color::new(255, 127, 127))
                        }
                    } else if *command == "list".to_string() {
                        print_color!("List volumes\n".to_string(), Color::new(127, 255, 127));
                    } else if *command == "dump".to_string() {
                        match args.get(1) {
                            Option::Some(arg) => {
                                let sector = arg.to_num();
                                print_color!("Dump sector: ".to_string() + sector, Color::new(127, 255, 127));

                                let data = zfs.read(sector, 1);
                                for i in 0..data.len() {
                                    if i % 32 == 0 {
                                        print!("\n".to_string() + String::from_num_radix(i, 16) + ":");
                                    }
                                    if let Option::Some(byte) = data.get(i) {
                                        print!(" ".to_string() + String::from_num_radix(*byte as usize, 16));
                                    }else{
                                        println!(" !".to_string());
                                    }
                                }
                                print!("\n".to_string());
                            },
                            Option::None => print_color!("No sector specified!\n".to_string(), Color::new(255, 127, 127))
                        }
                    }else if *command == "close".to_string() {
                        print_color!("Closing\n".to_string(), Color::new(255, 127, 127));
                        close = true;
                    } else {
                        print_color!("Commands: uber list dump close\n".to_string(), Color::new(127, 127, 255));
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
