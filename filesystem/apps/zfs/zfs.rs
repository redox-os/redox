//To use this, please install zfs-fuse
use redox::*;

use self::dnode::{DNodePhys, ObjectSetPhys};
use self::block_ptr::BlockPtr;
use self::dsl_dataset::DslDatasetPhys;
use self::dsl_dir::DslDirPhys;
use self::dvaddr::DVAddr;
use self::from_bytes::FromBytes;
use self::uberblock::Uberblock;
use self::vdev::VdevLabel;

pub mod block_ptr;
pub mod dnode;
pub mod dsl_dataset;
pub mod dsl_dir;
pub mod dvaddr;
pub mod from_bytes;
pub mod lzjb;
pub mod nvpair;
pub mod nvstream;
pub mod uberblock;
pub mod vdev;
pub mod xdr;
pub mod zap;
pub mod zil_header;

pub struct ZfsReader {
    disk: File,
}

impl ZfsReader {
    //TODO: Error handling
    pub fn read(&mut self, start: usize, length: usize) -> Vec<u8> {
        let mut ret: Vec<u8> = vec![0; length*512];

        self.disk.seek(SeekFrom::Start(start * 512));
        self.disk.read(&mut ret);

        return ret;
    }

    pub fn write(&mut self, block: usize, data: &[u8; 512]) {
        self.disk.seek(SeekFrom::Start(block * 512));
        self.disk.write(data);
    }

    pub fn read_dva(&mut self, dva: &DVAddr) -> Vec<u8> {
        self.read(dva.sector() as usize, dva.asize() as usize)
    }

    pub fn read_block(&mut self, block_ptr: &BlockPtr) -> Option<Vec<u8>> {
        let data = self.read_dva(&block_ptr.dvas[0]);
        match block_ptr.compression() {
            2 => {
                // compression off
                Some(data)
            },
            1 | 3 => {
                // lzjb compression
                let mut decompressed = vec![0; (block_ptr.lsize()*512) as usize];
                lzjb::decompress(&data, &mut decompressed);
                Some(decompressed)
            },
            _ => None,
        }
    }

    pub fn read_type<T: FromBytes>(&mut self, block_ptr: &BlockPtr) -> Option<T> {
        self.read_type_array(block_ptr, 0)
    }

    pub fn read_type_array<T: FromBytes>(&mut self, block_ptr: &BlockPtr, offset: usize) -> Option<T> {
        let data = self.read_block(block_ptr);
        data.and_then(|data| T::from_bytes(&data[offset*mem::size_of::<T>()..]))
    }

    pub fn uber(&mut self) -> Option<Uberblock> {
        let mut newest_uberblock: Option<Uberblock> = None;
        for i in 0..128 {
            match Uberblock::from_bytes(&self.read(256 + i * 2, 2)) {
                Some(uberblock) => {
                    let mut newest = false;
                    match newest_uberblock {
                        Some(previous) => {
                            if uberblock.txg > previous.txg {
                                newest = true;
                            }
                        }
                        None => newest = true,
                    }

                    if newest {
                        newest_uberblock = Some(uberblock);
                    }
                }
                None => (), //Invalid uberblock
            }
        }
        return newest_uberblock;
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum ZfsTraverse {
    ThisDir,
    Done,
}

pub struct ZFS {
    pub reader: ZfsReader,
    pub uberblock: Uberblock, // The active uberblock
    fs_objset: ObjectSetPhys,
    master_node: DNodePhys,
    root: u64,
}

impl ZFS {
    pub fn new(disk: File) -> Option<Self> {
        let mut zfs_reader = ZfsReader { disk: disk };

        let uberblock = zfs_reader.uber().unwrap();

        //let mos_dva = uberblock.rootbp.dvas[0];
        let mos: ObjectSetPhys = zfs_reader.read_type(&uberblock.rootbp).unwrap();
        let mos_block_ptr1 = mos.meta_dnode.get_blockptr(0);
        //let mos_block_ptr2 = mos.meta_dnode.get_blockptr(1);
        //let mos_block_ptr2 = mos.meta_dnode.get_blockptr(2);

        // 2nd dnode in MOS points at the root dataset zap
        let dnode1: DNodePhys = zfs_reader.read_type_array(&mos_block_ptr1, 1).unwrap();

        let root_ds: zap::MZapWrapper = zfs_reader.read_type(dnode1.get_blockptr(0)).unwrap();

        let root_ds_dnode: DNodePhys =
            zfs_reader.read_type_array(&mos_block_ptr1, root_ds.chunks[0].value as usize).unwrap();

        let dsl_dir = DslDirPhys::from_bytes(root_ds_dnode.get_bonus()).unwrap();
        let head_ds_dnode: DNodePhys =
            zfs_reader.read_type_array(&mos_block_ptr1, dsl_dir.head_dataset_obj as usize).unwrap();

        let root_dataset = DslDatasetPhys::from_bytes(head_ds_dnode.get_bonus()).unwrap();

        let fs_objset: ObjectSetPhys = zfs_reader.read_type(&root_dataset.bp).unwrap();

        let mut indirect: BlockPtr = zfs_reader.read_type_array(fs_objset.meta_dnode.get_blockptr(0), 0).unwrap();
        while indirect.level() > 0 {
            indirect = zfs_reader.read_type_array(&indirect, 0).unwrap();
        }

        // Master node is always the second object in the object set
        let master_node: DNodePhys = zfs_reader.read_type_array(&indirect, 1).unwrap();
        let master_node_zap: zap::MZapWrapper = zfs_reader.read_type(master_node.get_blockptr(0)).unwrap();
        // Find the ROOT zap entry
        let mut root = None;
        for chunk in &master_node_zap.chunks {
            if chunk.name().unwrap() == "ROOT" {
                root = Some(chunk.value);
            }
        }

        Some(ZFS {
            reader: zfs_reader,
            uberblock: uberblock,
            fs_objset: fs_objset,
            master_node: master_node,
            root: root.unwrap(),
        })
    }

    pub fn traverse<F, T>(&mut self, mut f: F) -> Option<T>
        where F: FnMut(&mut ZFS, &str, usize, &mut DNodePhys, &BlockPtr, &mut Option<T>) -> Option<ZfsTraverse>,
    {
        // Given the fs_objset and the object id of the root directory, we can traverse the
        // directory tree.
        // TODO: Cache object id of paths
        // TODO: Calculate path through objset blockptr tree to use
        let mut indirect: BlockPtr = self.reader.read_type_array(self.fs_objset.meta_dnode.get_blockptr(0), 0).unwrap();
        while indirect.level() > 0 {
            indirect = self.reader.read_type_array(&indirect, 0).unwrap();
        }
        // Set the cur_node to the root node, located at an L0 indirect block
        let root = self.root as usize;
        let mut cur_node: DNodePhys = self.reader.read_type_array(&indirect,
                                                                  self.root as usize).unwrap();
        let mut result = None;
        if f(self, "", root, &mut cur_node, &indirect, &mut result) == Some(ZfsTraverse::Done) {
            return result;
        }
        'traverse: loop {
            // Directory dnodes point at zap objects. File/directory names are mapped to their
            // fs_objset object ids.
            let dir_contents: zap::MZapWrapper = self.reader.read_type(cur_node.get_blockptr(0)).unwrap();
            let mut next_dir = None;
            for chunk in &dir_contents.chunks {
                match chunk.name() {
                    Some(chunk_name) => {
                        // Stop once we get to a null entry
                        if chunk_name.is_empty() {
                            break;
                        }

                        let traverse = f(self, chunk_name, chunk.value as usize,
                                         &mut cur_node, &indirect, &mut result);
                        if let Some(traverse) = traverse {
                            match traverse {
                                ZfsTraverse::ThisDir => {
                                    // Found the folder we were looking for
                                    next_dir = Some(chunk.value);
                                    break;
                                },
                                ZfsTraverse::Done => {
                                    break 'traverse;
                                },
                            }
                        }
                    },
                    None => {
                        // Invalid directory name
                        return None;
                    },
                }
            }
            if next_dir.is_none() {
                break;
            }
        }
        result
    }

    pub fn read_file(&mut self, path: &str) -> Option<Vec<u8>> {
        let path = path.trim_matches('/'); // Robust against different url styles
        let path_end_index = path.rfind('/').map(|i| i+1).unwrap_or(0);
        let path_end = &path[path_end_index..];
        let mut folder_iter = path.split('/');
        let mut folder = folder_iter.next();

        let file_contents =
            self.traverse(|zfs, name, node_id, node, indirect, result| {
                let mut this_dir = false;
                if let Some(folder) = folder {
                    if name == folder {
                        *node = zfs.reader.read_type_array(indirect,
                                                           node_id as usize).unwrap();
                        if name == path_end {
                            if node.object_type != 0x13 {
                                // Not a file
                                return Some(ZfsTraverse::Done);
                            }
                            // Found the file
                            let file_contents = zfs.reader.read_block(node.get_blockptr(0)).unwrap();
                            // TODO: Read file size from ZPL rather than look for terminating 0
                            let file_contents: Vec<u8> = file_contents.into_iter().take_while(|c| *c != 0).collect();
                            *result = Some(file_contents);
                            return Some(ZfsTraverse::Done);
                        }
                        this_dir = true;
                    }
                }
                if this_dir {
                    if node.object_type != 0x14 {
                        // Not a folder
                        return Some(ZfsTraverse::Done);
                    }
                    folder = folder_iter.next();
                    return Some(ZfsTraverse::ThisDir);
                }
                None
            });

        file_contents
    }

    pub fn ls(&mut self, path: &str) -> Option<Vec<String>> {
        let path = path.trim_matches('/'); // Robust against different url styles
        let path_end_index = path.rfind('/').map(|i| i+1).unwrap_or(0);
        let path_end = &path[path_end_index..];
        let mut folder_iter = path.split('/');
        let mut folder = folder_iter.next();

        let file_contents =
            self.traverse(|zfs, name, node_id, node, indirect, result| {
                let mut this_dir = false;
                if let Some(folder) = folder {
                    if name == folder {
                        if folder == path_end {
                            *node = zfs.reader.read_type_array(indirect,
                                                               node_id as usize).unwrap();
                            let dir_contents: zap::MZapWrapper = zfs.reader
                                                                    .read_type(node.get_blockptr(0))
                                                                    .unwrap();

                            let ls: Vec<String> =
                                dir_contents.chunks
                                            .iter()
                                            .map(|x| {
                                                if x.value & 0xF000000000000000 == 0x4000000000000000 {
                                                    x.name().unwrap().to_string() + "/"
                                                } else {
                                                    x.name().unwrap().to_string()
                                                }
                                            })
                                            .take_while(|x| !x.is_empty())
                                            .collect();
                            *result = Some(ls);
                            return Some(ZfsTraverse::Done);
                        }
                        this_dir = true;
                    }
                }
                if this_dir {
                    folder = folder_iter.next();
                    return Some(ZfsTraverse::ThisDir);
                }
                None
            });

        file_contents
    }
}

//TODO: Find a way to remove all the to_string's
pub fn main() {
    console_title("ZFS");

    let red = Color::rgba(255, 127, 127, 255);
    let green = Color::rgba(127, 255, 127, 255);
    let blue = Color::rgba(127, 127, 255, 255);

    println!("Type open zfs.img to open the image file");

    let mut zfs_option: Option<ZFS> = None;

    while let Some(line) = readln!() {
        let mut args: Vec<String> = Vec::new();
        for arg in line.split(' ') {
            args.push(arg.to_string());
        }

        if let Some(command) = args.get(0) {
            println!("# {}", line);

            let mut close = false;
            match zfs_option {
                Some(ref mut zfs) => {
                    if command == "uber" {
                        let ref uberblock = zfs.uberblock;
                        //128 KB of ubers after 128 KB of other stuff
                        println_color!(green, "Newest Uberblock {:X}", zfs.uberblock.magic);
                        println!("Version {}", uberblock.version);
                        println!("TXG {}", uberblock.txg);
                        println!("GUID {:X}", uberblock.guid_sum);
                        println!("Timestamp {}", uberblock.timestamp);
                        println!("ROOTBP[0] {:?}", uberblock.rootbp.dvas[0]);
                        println!("ROOTBP[1] {:?}", uberblock.rootbp.dvas[1]);
                        println!("ROOTBP[2] {:?}", uberblock.rootbp.dvas[2]);
                    } else if command == "vdev_label" {
                        match VdevLabel::from_bytes(&zfs.reader.read(0, 256 * 2)) {
                            Some(ref mut vdev_label) => {
                                let mut xdr = xdr::MemOps::new(&mut vdev_label.nv_pairs);
                                let nv_list = nvstream::decode_nv_list(&mut xdr);
                                println_color!(green, "Got nv_list:\n{:?}", nv_list);
                            },
                            None => { println_color!(red, "Couldn't read vdev_label"); },
                        }
                    } else if command == "file" {
                        match args.get(1) {
                            Some(arg) => {
                                let file = zfs.read_file(arg);
                                match file {
                                    Some(file) => {
                                        println!("File contents: {}", str::from_utf8(&file).unwrap());
                                    },
                                    None => println_color!(red, "Failed to read file"),
                                }
                            }
                            None => println_color!(red, "Usage: file <path>"),
                        }
                    } else if command == "ls" {
                        match args.get(1) {
                            Some(arg) => {
                                let ls = zfs.ls(arg);
                                match ls {
                                    Some(ls) => {
                                        for item in &ls {
                                            print!("{}\t", item);
                                        }
                                    },
                                    None => println_color!(red, "Failed to read directory"),
                                }
                            }
                            None => println_color!(red, "Usage: ls <path>"),
                        }
                    } else if command == "mos" {
                        let ref uberblock = zfs.uberblock;
                        let mos_dva = uberblock.rootbp.dvas[0];
                        println_color!(green, "DVA: {:?}", mos_dva);
                        println_color!(green, "type: {:X}", uberblock.rootbp.object_type());
                        println_color!(green, "checksum: {:X}", uberblock.rootbp.checksum());
                        println_color!(green, "compression: {:X}", uberblock.rootbp.compression());
                        println!("Reading {} sectors starting at {}", mos_dva.asize(), mos_dva.sector());
                        let obj_set: Option<ObjectSetPhys> = zfs.reader.read_type(&uberblock.rootbp);
                        if let Some(ref obj_set) = obj_set {
                            println_color!(green, "Got meta object set");
                            println_color!(green, "os_type: {:X}", obj_set.os_type);
                            println_color!(green, "meta dnode: {:?}\n", obj_set.meta_dnode);

                            println_color!(green, "Reading MOS...");
                            let mos_block_ptr = obj_set.meta_dnode.get_blockptr(0);
                            let mos_array_dva = mos_block_ptr.dvas[0];

                            println_color!(green, "DVA: {:?}", mos_array_dva);
                            println_color!(green, "type: {:X}", mos_block_ptr.object_type());
                            println_color!(green, "checksum: {:X}", mos_block_ptr.checksum());
                            println_color!(green, "compression: {:X}", mos_block_ptr.compression());
                            println!("Reading {} sectors starting at {}", mos_array_dva.asize(), mos_array_dva.sector());
                            let dnode: Option<DNodePhys> = zfs.reader.read_type_array(&mos_block_ptr, 1);
                            println_color!(green, "Got MOS dnode array");
                            println_color!(green, "dnode: {:?}", dnode);

                            if let Some(ref dnode) = dnode {
                                println_color!(green, "Reading object directory zap object...");
                                let zap_obj: Option<zap::MZapWrapper> = zfs.reader.read_type(dnode.get_blockptr(0));
                                println!("{:?}", zap_obj);
                            }
                        }
                    } else if command == "dump" {
                        match args.get(1) {
                            Some(arg) => {
                                let sector = arg.to_num();
                                println_color!(green, "Dump sector: {}", sector);

                                let data = zfs.reader.read(sector, 1);
                                for i in 0..data.len() {
                                    if i % 32 == 0 {
                                        print!("\n{:X}:", i);
                                    }
                                    if let Some(byte) = data.get(i) {
                                        print!(" {:X}", *byte);
                                    } else {
                                        println!(" !");
                                    }
                                }
                                print!("\n");
                            }
                            None => println_color!(red, "No sector specified!"),
                        }
                    } else if command == "close" {
                        println_color!(red, "Closing");
                        close = true;
                    } else {
                        println_color!(blue, "Commands: uber vdev_label mos file ls dump close");
                    }
                }
                None => {
                    if command == "open" {
                        match args.get(1) {
                            Some(arg) => {
                                match File::open(arg) {
                                    Some(file) => {
                                        println_color!(green, "Open: {}", arg);
                                        zfs_option = ZFS::new(file);
                                    },
                                    None => println_color!(red, "File not found!"),
                                }
                            }
                            None => println_color!(red, "No file specified!"),
                        }
                    } else {
                        println_color!(blue, "Commands: open");
                    }
                }
            }
            if close {
                zfs_option = None;
            }
        }
    }
}
