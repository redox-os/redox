use common::memory::*;
use common::scheduler::*;

use drivers::disk::*;

use programs::common::*;

#[derive(Copy, Clone)]
#[repr(packed)]
pub struct Extent{
    pub block: u64,
    pub length: u64
}

#[repr(packed)]
pub struct Header {
    pub signature: [u8; 8],
    pub version: u32,
    pub name: [u8; 244],
    pub extents: [Extent; 16]
}

#[repr(packed)]
pub struct Node {
    pub name: [u8; 256],
    pub extents: [Extent; 16]
}

impl Clone for Node {
    fn clone(&self) -> Node {
        return Node {
            name: self.name,
            extents: self.extents
        };
    }
}

pub struct FileSystem {
    pub disk: Disk,
    pub header: Header,
    pub nodes: Vec<Node>
}

impl FileSystem {
    pub fn from_disk(disk: Disk) -> FileSystem{
        unsafe{
            let header_ptr: *const Header = alloc_type();
            disk.read(1, 1, header_ptr as usize);
            let header = ptr::read(header_ptr);
            unalloc(header_ptr as usize);

            let mut nodes = Vec::new();
            let node_ptr: *const Node = alloc_type();
            for extent in &header.extents {
                if extent.block > 0 {
                    for node_address in extent.block..extent.block + (extent.length + 511)/512 {
                        disk.read(node_address, 1, node_ptr as usize);
                        nodes.push(ptr::read(node_ptr));
                    }
                }
            }
            unalloc(node_ptr as usize);

            return FileSystem {
                disk:disk,
                header: header,
                nodes: nodes
            };
        }
    }

    pub fn valid(&self) -> bool {
        return self.header.signature[0] == 'R' as u8
            && self.header.signature[1] == 'E' as u8
            && self.header.signature[2] == 'D' as u8
            && self.header.signature[3] == 'O' as u8
            && self.header.signature[4] == 'X' as u8
            && self.header.signature[5] == 'F' as u8
            && self.header.signature[6] == 'S' as u8
            && self.header.signature[7] == '\0' as u8
            && self.header.version == 0xFFFFFFFF;
    }

    pub fn node(&self, filename: &String) -> Option<Node>{
        for node in self.nodes.iter() {
            if String::from_c_slice(&node.name) == *filename {
                return Option::Some(node.clone());
            }
        }

        return Option::None;
    }

    pub fn list(&self, directory: &String) -> Vec<String> {
        let mut ret = Vec::<String>::new();

        for node in self.nodes.iter() {
            let node_name = String::from_c_slice(&node.name);
            if node_name.starts_with(directory.clone()) {
                ret.push(node_name.substr(directory.len(), node_name.len() - directory.len()));
            }
        }

        return ret;
    }
}

pub struct FileResource {
    pub node: Node,
    pub vec: Vec<u8>,
    pub seek: usize
}

impl Resource for FileResource {
    fn url(&self) -> URL {
        return URL::from_string(&String::from_c_slice(&self.node.name));
    }

    fn stat(&self) -> ResourceType {
        return ResourceType::File;
    }

    fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        let mut i = 0;
        while i < buf.len() && self.seek < self.vec.len() {
            match self.vec.get(self.seek) {
                Option::Some(b) => buf[i] = *b,
                Option::None => ()
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
        return Option::Some(i);
    }

    fn seek(&mut self, pos: ResourceSeek) -> Option<usize> {
        match pos {
            ResourceSeek::Start(offset) => self.seek = offset,
            ResourceSeek::Current(offset) => self.seek = max(0, self.seek as isize + offset) as usize,
            ResourceSeek::End(offset) => self.seek = max(0, self.vec.len() as isize + offset) as usize
        }
        while self.vec.len() < self.seek {
            self.vec.push(0);
        }
        return Option::Some(self.seek);
    }

    fn flush(&mut self) -> bool {
        return false;
    }
}

pub struct FileScheme {
    pub fs: FileSystem
}

impl SessionItem for FileScheme {
    fn scheme(&self) -> String {
        return "file".to_string();
    }

    fn open(&mut self, url: &URL) -> Box<Resource>{
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
                        }else{
                            line = dirname.clone();
                            dirs.push(dirname);
                        }
                    },
                    Option::None => line = file.clone()
                }
                if line.len() > 0 {
                    if list.len() > 0 {
                        list = list + '\n' + line;
                    }else{
                        list = line;
                    }
                }
            }

            return box VecResource::new(url.clone(), ResourceType::Dir, list.to_utf8());
        }else{
            match self.fs.node(&path) {
                Option::Some(node) => {
                    let mut vec: Vec<u8> = Vec::new();
                    if node.extents[0].block > 0 && node.extents[0].length > 0 {
                        unsafe {
                            let data = alloc(node.extents[0].length as usize);
                            if data > 0 {
                                let reenable = start_no_ints();

                                self.fs.disk.read(node.extents[0].block, ((node.extents[0].length + 511)/512) as u16, data);

                                end_no_ints(reenable);

                                vec = Vec {
                                    data: data as *mut u8,
                                    length: node.extents[0].length as usize
                                };
                            }
                        }
                    }

                    return box FileResource {
                        node: node,
                        vec: vec,
                        seek: 0
                    };
                },
                Option::None => return box NoneResource
            }
        }
    }
}
