use alloc::boxed::Box;

use core::{cmp, ptr};

use drivers::disk::*;

use common::debug;
use common::memory;
use common::resource::{NoneResource, Resource, ResourceSeek, ResourceType, URL, VecResource};
use common::scheduler::*;
use common::string::{String, ToString};
use common::vec::Vec;

use programs::common::SessionItem;

#[derive(Copy, Clone)]
#[repr(packed)]
pub struct Extent {
    pub block: u64,
    pub length: u64,
}

#[repr(packed)]
pub struct Header {
    pub signature: [u8; 8],
    pub version: u32,
    pub name: [u8; 244],
    pub extents: [Extent; 16],
}

#[repr(packed)]
pub struct NodeData {
    pub name: [u8; 256],
    pub extents: [Extent; 16],
}

pub struct Node {
    pub address: u64,
    pub name: String,
    pub extents: [Extent; 16],
}

impl Node {
    pub fn new(address: u64, data: NodeData) -> Node {
        let mut utf8: Vec<u8> = Vec::new();
        for i in 0..data.name.len() {
            let c = data.name[i];
            if c == 0 {
                break;
            } else {
                utf8.push(c);
            }
        }

        Node {
            address: address,
            name: String::from_utf8(&utf8),
            extents: data.extents,
        }
    }
}

impl Clone for Node {
    fn clone(&self) -> Node {
        Node {
            address: self.address,
            name: self.name.clone(),
            extents: self.extents,
        }
    }
}

pub struct FileSystem {
    pub disk: Disk,
    pub header: Header,
    pub nodes: Vec<Node>,
}

impl FileSystem {
    pub fn from_disk(disk: Disk) -> FileSystem {
        unsafe {
            let header_ptr: *const Header = memory::alloc_type();
            disk.read(1, 1, header_ptr as usize);
            let header = ptr::read(header_ptr);
            memory::unalloc(header_ptr as usize);

            let mut nodes = Vec::new();
            let node_data: *const NodeData = memory::alloc_type();
            for extent in &header.extents {
                if extent.block > 0 {
                    for node_address in extent.block..extent.block + (extent.length + 511) / 512 {
                        disk.read(node_address, 1, node_data as usize);

                        nodes.push(Node::new(node_address, ptr::read(node_data)));
                    }
                }
            }
            memory::unalloc(node_data as usize);

            return FileSystem {
                disk: disk,
                header: header,
                nodes: nodes,
            };
        }
    }

    pub fn valid(&self) -> bool {
        return self.header.signature[0] == 'R' as u8 && self.header.signature[1] == 'E' as u8 &&
               self.header.signature[2] == 'D' as u8 &&
               self.header.signature[3] == 'O' as u8 &&
               self.header.signature[4] == 'X' as u8 &&
               self.header.signature[5] == 'F' as u8 &&
               self.header.signature[6] == 'S' as u8 &&
               self.header.signature[7] == '\0' as u8 &&
               self.header.version == 0xFFFFFFFF;
    }

    pub fn node(&self, filename: &String) -> Option<Node> {
        for node in self.nodes.iter() {
            if node.name == *filename {
                return Option::Some(node.clone());
            }
        }

        return Option::None;
    }

    pub fn list(&self, directory: &String) -> Vec<String> {
        let mut ret = Vec::<String>::new();

        for node in self.nodes.iter() {
            if node.name.starts_with(directory.clone()) {
                ret.push(node.name.substr(directory.len(), node.name.len() - directory.len()));
            }
        }

        return ret;
    }
}

pub struct FileResource {
    pub disk: Disk,
    pub node: Node,
    pub vec: Vec<u8>,
    pub seek: usize,
    pub dirty: bool,
}

impl Resource for FileResource {
    fn url(&self) -> URL {
        return URL::from_string(&("file:///".to_string() + &self.node.name));
    }

    fn stat(&self) -> ResourceType {
        return ResourceType::File;
    }

    fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        let mut i = 0;
        while i < buf.len() && self.seek < self.vec.len() {
            match self.vec.get(self.seek) {
                Option::Some(b) => buf[i] = *b,
                Option::None => (),
            }
            self.seek += 1;
            i += 1;
        }
        return Option::Some(i);
    }

    fn write(&mut self, buf: &[u8]) -> Option<usize> {
        let mut i = 0;
        while i < buf.len() && self.seek < self.vec.len() {
            self.vec.set(self.seek, buf[i]);
            self.seek += 1;
            i += 1;
        }
        while i < buf.len() {
            self.vec.push(buf[i]);
            self.seek += 1;
            i += 1;
        }
        if i > 0 {
            self.dirty = true;
        }
        return Option::Some(i);
    }

    fn seek(&mut self, pos: ResourceSeek) -> Option<usize> {
        match pos {
            ResourceSeek::Start(offset) => self.seek = offset,
            ResourceSeek::Current(offset) =>
                self.seek = cmp::max(0, self.seek as isize + offset) as usize,
            ResourceSeek::End(offset) =>
                self.seek = cmp::max(0, self.vec.len() as isize + offset) as usize,
        }
        while self.vec.len() < self.seek {
            self.vec.push(0);
        }
        return Option::Some(self.seek);
    }

    // TODO: Rename to sync
    // TODO: Check to make sure proper amount of bytes written. See Disk::write
    // TODO: Allow reallocation
    fn sync(&mut self) -> bool {
        if self.dirty {
            let block_size: usize = 512;

            let mut node_dirty = false;
            let mut pos: isize = 0;
            let mut remaining = self.vec.len() as isize;
            for ref mut extent in &mut self.node.extents {
                //Make sure it is a valid extent
                if extent.block > 0 && extent.length > 0 {
                    let current_sectors = (extent.length as usize + block_size - 1) / block_size;
                    let max_size = current_sectors * 512;

                    let size = cmp::min(remaining as usize, max_size);
                    let sectors = (size + block_size - 1) / block_size;

                    if size as u64 != extent.length {
                        extent.length = size as u64;
                        node_dirty = true;
                    }

                    unsafe {
                        let data = self.vec.as_ptr().offset(pos) as usize;
                        //TODO: Make sure data is copied safely into an zeroed area of the right size!

                        let mut sector: usize = 0;
                        while sectors - sector >= 65536 {
                            self.disk.read(extent.block + sector as u64,
                                          65535,
                                          data + sector * 512);
                            sector += 65535;
                        }
                        if sector < sectors {
                            self.disk.read(extent.block + sector as u64,
                                          (sectors - sector) as u16,
                                          data + sector * 512);
                        }
                    }

                    pos += size as isize;
                    remaining -= size as isize;
                }
            }

            if node_dirty {
                debug::d("Node dirty, should rewrite\n");
            }

            self.dirty = false;

            if remaining > 0 {
                debug::d("Need to reallocate file, extra: ");
                debug::ds(remaining);
                debug::dl();
                return false;
            }
        }
        return true;
    }
}

impl Drop for FileResource {
    fn drop(&mut self) {
        self.sync();
    }
}

pub struct FileScheme {
    pub fs: FileSystem,
}

impl SessionItem for FileScheme {
    fn scheme(&self) -> String {
        return "file".to_string();
    }

    fn open(&mut self, url: &URL) -> Box<Resource> {
        let path = url.path();
        if path.len() == 0 || path.ends_with("/".to_string()) {
            let mut list = String::new();
            let mut dirs: Vec<String> = Vec::new();

            for file in self.fs.list(&path).iter() {
                let line;
                match file.find("/".to_string()) {
                    Option::Some(index) => {
                        let dirname = file.substr(0, index + 1);
                        let mut found = false;
                        for dir in dirs.iter() {
                            if dirname == *dir {
                                found = true;
                                break;
                            }
                        }
                        if found {
                            line = String::new();
                        } else {
                            line = dirname.clone();
                            dirs.push(dirname);
                        }
                    }
                    Option::None => line = file.clone(),
                }
                if line.len() > 0 {
                    if list.len() > 0 {
                        list = list + '\n' + line;
                    } else {
                        list = line;
                    }
                }
            }

            return box VecResource::new(url.clone(), ResourceType::Dir, list.to_utf8());
        } else {
            match self.fs.node(&path) {
                Option::Some(node) => {
                    let mut vec: Vec<u8> = Vec::new();
                    //TODO: Handle more extents
                    for extent in &node.extents {
                        if extent.block > 0 && extent.length > 0 {
                            unsafe {
                                let data = memory::alloc(extent.length as usize);
                                if data > 0 {
                                    let reenable = start_no_ints();

                                    let sectors = (extent.length as usize + 511)/512;
                                    let mut sector: usize = 0;
                                    while sectors - sector >= 65536 {
                                        self.fs.disk.read(extent.block + sector as u64,
                                                      65535,
                                                      data + sector * 512);
                                        sector += 65535;
                                    }
                                    if sector < sectors {
                                        self.fs.disk.read(extent.block + sector as u64,
                                                      (sectors - sector) as u16,
                                                      data + sector * 512);
                                    }

                                    end_no_ints(reenable);

                                    vec.push_all(&Vec {
                                        data: data as *mut u8,
                                        length: extent.length as usize,
                                    });
                                }
                            }
                        }
                    }

                    return box FileResource {
                        disk: self.fs.disk,
                        node: node,
                        vec: vec,
                        seek: 0,
                        dirty: false,
                    };
                }
                Option::None => {
                    /*
                    d("Creating ");
                    path.d();
                    dl();

                    let mut name = [0; 256];
                    for i in 0..256 {
                        //TODO: UTF8
                        let b = path[i] as u8;
                        name[i] = b;
                        if b == 0 {
                            break;
                        }
                    }

                    let node = Node {
                        name: name,
                        extents: [Extent { block: 0, length: 0 }; 16]
                    };

                    //TODO: Sync to disk
                    let mut node_i = 0;
                    while node_i < self.fs.nodes.len() {
                        let mut cmp = 0;

                        if let Option::Some(other_node) = self.fs.nodes.get(node_i) {
                            for i in 0..256 {
                                if other_node.name[i] != node.name[i] {
                                    cmp = other_node.name[i] as isize - node.name[i] as isize;
                                    break;
                                }
                            }
                        }

                        if cmp >= 0 {
                            break;
                        }

                        node_i += 1;
                    }
                    d("Insert at ");
                    dd(node_i);
                    dl();
                    self.fs.nodes.insert(node_i, node.clone());

                    return box FileResource {
                        node: node,
                        vec: Vec::new(),
                        seek: 0
                    };
                    */
                    return box NoneResource;
                }
            }
        }
    }
}
