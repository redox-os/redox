//To use this, please install zfs-fuse
use redox::*;

mod nvpair;
mod nvstream;
mod xdr;

pub struct ZFS {
    disk: File,
}

impl ZFS {
    pub fn new(disk: File) -> Self {
        ZFS { disk: disk }
    }

    //TODO: Error handling
    pub fn read(&mut self, start: usize, length: usize) -> Vec<u8> {
        let mut ret: Vec<u8> = vec![0; length*512];

        self.disk.seek(Seek::Start(start * 512));
        self.disk.read(&mut ret);

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
    pub uberblocks: [Uberblock; 128],
}

impl VdevLabel {
    pub fn from(data: &[u8]) -> Option<Self> {
        if data.len() >= 262144 {
            let vdev_label = unsafe { ptr::read(data.as_ptr() as *const VdevLabel) };
            Some(vdev_label)
        } else {
            Option::None
        }
    }
}

#[derive(Copy, Clone)]
#[repr(packed)]
pub struct Uberblock {
    pub magic: u64,
    pub version: u64,
    pub txg: u64,
    pub guid_sum: u64,
    pub timestamp: u64,
    pub rootbp: BlockPtr,
}

impl Uberblock {
    pub fn magic_little() -> u64 {
        return 0x0cb1ba00;
    }

    pub fn magic_big() -> u64 {
        return 0x00bab10c;
    }

    pub fn from(data: &Vec<u8>) -> Option<Self> {
        if data.len() >= 1024 {
            let uberblock = unsafe { ptr::read(data.as_ptr() as *const Uberblock) };
            if uberblock.magic == Uberblock::magic_little() {
                println!("Little Magic");
                return Option::Some(uberblock);
            } else if uberblock.magic == Uberblock::magic_big() {
                println!("Big Magic");
                return Option::Some(uberblock);
            } else if uberblock.magic > 0 {
                println!("Unknown Magic: {:X}", uberblock.magic as usize);
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
    pub checksum: u64,
}

impl Gang {
    pub fn magic() -> u64 {
        return 0x117a0cb17ada1002;
    }
}

//TODO: Find a way to remove all the to_string's
pub fn main() {
    console_title("ZFS");

    let red = [255, 127, 127, 255];
    let green = [127, 255, 127, 255];
    let blue = [127, 127, 255, 255];

    println!("Type open zfs.img to open the image file");

    let mut zfs_option: Option<ZFS> = Option::None;

    while let Option::Some(line) = readln!() {
        let mut args: Vec<String> = Vec::new();
        for arg in line.split(' ') {
            args.push(arg.to_string());
        }

        if let Option::Some(command) = args.get(0) {
            println!("# {}", line);

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
                                        }
                                        Option::None => newest = true,
                                    }

                                    if newest {
                                        newest_uberblock = Option::Some(uberblock);
                                    }
                                }
                                Option::None => (), //Invalid uberblock
                            }
                        }

                        match newest_uberblock {
                            Option::Some(uberblock) => {
                                println_color!(green, "Newest Uberblock");
                                //TODO: Do not use as usize
                                println!("Magic: {:X}", uberblock.magic as usize);
                                println!("Version: {}", uberblock.version as usize);
                                println!("TXG: {}", uberblock.txg as usize);
                                println!("Timestamp: {}", uberblock.timestamp as usize);
                                println!("MOS: {}",
                                         uberblock.rootbp.dvas[0].sector() as usize);
                            }
                            Option::None => println_color!(red, "No valid uberblock found!"),
                        }
                    } else if *command == "vdev_label".to_string() {
                        let mut vdev_label = VdevLabel::from(&zfs.read(0, 256 * 2)); // 256KB of vdev label
                        match vdev_label {
                            Some(ref mut vdev_label) => {
                                let mut xdr = xdr::MemOps::new(&mut vdev_label.nv_pairs);
                                let nv_list = nvstream::decode_nv_list(&mut xdr);
                                println_color!(green, "Got nv_list:\n{:?}", nv_list);
                            },
                            None => { println_color!(red, "Couldn't read vdev_label"); },
                        }
                    } else if *command == "list".to_string() {
                        println_color!(green, "List volumes");
                    } else if *command == "dump".to_string() {
                        match args.get(1) {
                            Option::Some(arg) => {
                                let sector = arg.to_num();
                                println_color!(green, "Dump sector: {}", sector);

                                let data = zfs.read(sector, 1);
                                for i in 0..data.len() {
                                    if i % 32 == 0 {
                                        print!("\n{:X}:", i);
                                    }
                                    if let Option::Some(byte) = data.get(i) {
                                        print!(" {:X}", *byte);
                                    } else {
                                        println!(" !");
                                    }
                                }
                                print!("\n");
                            }
                            Option::None => println_color!(red, "No sector specified!"),
                        }
                    } else if *command == "close".to_string() {
                        println_color!(red, "Closing");
                        close = true;
                    } else {
                        println_color!(blue, "Commands: uber vdev_label list dump close");
                    }
                }
                Option::None => {
                    if *command == "open".to_string() {
                        match args.get(1) {
                            Option::Some(arg) => {
                                println_color!(green, "Open: {}", arg);
                                zfs_option = Option::Some(ZFS::new(File::open(arg)));
                            }
                            Option::None => println_color!(red, "No file specified!"),
                        }
                    } else {
                        println_color!(blue, "Commands: open");
                    }
                }
            }
            if close {
                zfs_option = Option::None;
            }
        }
    }
}
