use alloc::arc::Arc;
use alloc::boxed::Box;

use core::{cmp, mem};
use core::sync::atomic::{AtomicBool, Ordering};

use drivers::disk::*;
use drivers::pciconfig::PCIConfig;

use common::context::context_switch;
use common::debug;
use common::memory::Memory;
use common::resource::{Resource, ResourceSeek, URL, VecResource};
use common::string::{String, ToString};
use common::vec::Vec;

use programs::session::SessionItem;

/// The header of the fs
#[repr(packed)]
pub struct Header {
    pub signature: [u8; 8],
    pub version: u32,
    pub name: [u8; 244],
    pub extents: [Extent; 16],
}

/// Data for a node
#[repr(packed)]
pub struct NodeData {
    pub name: [u8; 256],
    pub extents: [Extent; 16],
}

/// A file node
pub struct Node {
    pub address: u64,
    pub name: [u8; 256],
    pub extents: [Extent; 16],
}

impl Node {
    /// Create a new file node from an address and some data
    pub fn new(address: u64, data: &NodeData) -> Self {
        Node {
            address: address,
            name: data.name,
            extents: data.extents,
        }
    }
}

impl Clone for Node {
    fn clone(&self) -> Self {
        let name: [u8; 256] = self.name;
        Node {
            address: self.address,
            name: name,
            extents: self.extents,
        }
    }
}

/// A file system
pub struct FileSystem {
    pub disk: Disk,
    pub header: Header,
    pub nodes: Vec<Node>,
}

impl FileSystem {
    /// Create a file system from a disk
    pub fn from_disk(mut disk: Disk) -> Option<Self> {
        unsafe {
            if disk.identify() {
                debug::d(" Disk Found");

                let header_ptr = Memory::<Header>::new(1).unwrap();
                disk.read(1, 1, header_ptr.address());
                let header = header_ptr.read(0);
                drop(header_ptr);

                if header.signature[0] == 'R' as u8 &&
                   header.signature[1] == 'E' as u8 &&
                   header.signature[2] == 'D' as u8 &&
                   header.signature[3] == 'O' as u8 &&
                   header.signature[4] == 'X' as u8 &&
                   header.signature[5] == 'F' as u8 &&
                   header.signature[6] == 'S' as u8 &&
                   header.signature[7] == '\0' as u8 &&
                   header.version == 0xFFFFFFFF {

                    debug::d(" Redox Filesystem\n");

                    let mut nodes = Vec::new();
                    for extent in &header.extents {
                        if extent.block > 0 && extent.length > 0 {
                            if let Some(mut data) = Memory::<NodeData>::new(extent.length as usize /
                                                           mem::size_of::<NodeData>()) {
                                let sectors = (extent.length as usize + 511) / 512;
                                let mut sector: usize = 0;
                                while sectors - sector >= 65536 {
                                    let request = Request {
                                        extent: Extent {
                                            block: extent.block + sector as u64,
                                            length: 65536 * 512,
                                        },
                                        mem: data.address() + sector * 512,
                                        read: true,
                                        complete: Arc::new(AtomicBool::new(false)),
                                    };

                                    disk.read(extent.block + sector as u64,
                                              0,
                                              data.address() + sector * 512);

                                    /*
                                    disk.request(request.clone());

                                    while request.complete.load(Ordering::SeqCst) == false {
                                        disk.on_poll();
                                    }
                                    */

                                    sector += 65535;
                                }
                                if sector < sectors {
                                    let request = Request {
                                        extent: Extent {
                                            block: extent.block + sector as u64,
                                            length: (sectors - sector) as u64 * 512,
                                        },
                                        mem: data.address() + sector * 512,
                                        read: true,
                                        complete: Arc::new(AtomicBool::new(false)),
                                    };

                                        disk.read(extent.block + sector as u64,
                                                  (sectors - sector) as u16,
                                                  data.address() + sector * 512);
                                        /*
                                        disk.request(request.clone());

                                        while request.complete.load(Ordering::SeqCst) == false {
                                            disk.on_poll();
                                        }
                                        */
                                }

                                for i in 0..extent.length as usize / mem::size_of::<NodeData>() {
                                    nodes.push(Node::new(extent.block + i as u64, &data[i]));
                                }
                            }
                        }
                    }

                    return Some(FileSystem {
                        disk: disk,
                        header: header,
                        nodes: nodes,
                    });
                } else {
                    debug::d(" Unknown Filesystem\n");
                }
            } else {
                debug::d(" Disk Not Found\n");
            }
        }

        None
    }

    /// Get node with a given filename
    pub fn node(&self, filename: &String) -> Option<Node> {
        for node in self.nodes.iter() {
            let node_name = String::from_c_slice(&node.name);
            if node_name == *filename {
                return Some(node.clone());
            }
        }

        None
    }

    /// List nodes in a given directory
    pub fn list(&self, directory: &String) -> Vec<String> {
        let mut ret = Vec::<String>::new();

        for node in self.nodes.iter() {
            let node_name = String::from_c_slice(&node.name);
            if node_name.starts_with(directory.clone()) {
                ret.push(node_name.substr(directory.len(), node_name.len() - directory.len()));
            }
        }

        ret
    }
}

/// A file resource
pub struct FileResource {
    pub scheme: *mut FileScheme,
    pub node: Node,
    pub vec: Vec<u8>,
    pub seek: usize,
    pub dirty: bool,
}

impl Resource for FileResource {
    fn dup(&self) -> Option<Box<Resource>> {
        Some(box FileResource {
            scheme: self.scheme,
            node: self.node.clone(),
            vec: self.vec.clone(),
            seek: self.seek,
            dirty: self.dirty,
        })
    }

    fn url(&self) -> URL {
        return URL::from_string(&("file:///".to_string() + &String::from_c_slice(&self.node.name)));
    }

    fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        let mut i = 0;
        while i < buf.len() && self.seek < self.vec.len() {
            match self.vec.get(self.seek) {
                Some(b) => buf[i] = *b,
                None => (),
            }
            self.seek += 1;
            i += 1;
        }
        Some(i)
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
        Some(i)
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
        Some(self.seek)
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

                    if size as u64 != extent.length {
                        extent.length = size as u64;
                        node_dirty = true;
                    }

                    unsafe {
                        let data = self.vec.as_ptr().offset(pos) as usize;
                        //TODO: Make sure data is copied safely into an zeroed area of the right size!

                        let sectors = (extent.length as usize + 511) / 512;
                        let mut sector: usize = 0;
                        while sectors - sector >= 65536 {
                            let request = Request {
                                extent: Extent {
                                    block: extent.block + sector as u64,
                                    length: 65536 * 512,
                                },
                                mem: data + sector * 512,
                                read: false,
                                complete: Arc::new(AtomicBool::new(false)),
                            };

                            (*self.scheme).fs.disk.request(request.clone());

                            while request.complete.load(Ordering::SeqCst) == false {
                                context_switch(false);
                            }

                            sector += 65535;
                        }
                        if sector < sectors {
                            let request = Request {
                                extent: Extent {
                                    block: extent.block + sector as u64,
                                    length: (sectors - sector) as u64 * 512,
                                },
                                mem: data + sector * 512,
                                read: false,
                                complete: Arc::new(AtomicBool::new(false)),
                            };

                            (*self.scheme).fs.disk.request(request.clone());

                            while request.complete.load(Ordering::SeqCst) == false {
                                context_switch(false);
                            }
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
        true
    }
}

impl Drop for FileResource {
    fn drop(&mut self) {
        self.sync();
    }
}

/// A file scheme (pci + fs)
pub struct FileScheme {
    pci: PCIConfig,
    fs: FileSystem,
}

impl FileScheme {
    ///TODO Allow busmaster for secondary
    /// Create a new file scheme from a PCI configuration
    pub fn new(mut pci: PCIConfig) -> Option<Box<Self>> {
        unsafe { pci.flag(4, 4, true) }; // Bus mastering

        let base = unsafe { pci.read(0x20) } as u16 & 0xFFF0;

        debug::d("IDE on ");
        debug::dh(base as usize);
        debug::dl();

        debug::d("Primary Master:");
        if let Some(fs) = FileSystem::from_disk(Disk::primary_master(base)) {
            return Some(box FileScheme {
                pci: pci,
                fs: fs,
            });
        }

        debug::d("Primary Slave:");
        if let Some(fs) = FileSystem::from_disk(Disk::primary_slave(base)) {
            return Some(box FileScheme {
                pci: pci,
                fs: fs,
            });
        }

        debug::d("Secondary Master:");
        if let Some(fs) = FileSystem::from_disk(Disk::secondary_master(base)) {
            return Some(box FileScheme {
                pci: pci,
                fs: fs,
            });
        }

        debug::d("Secondary Slave:");
        if let Some(fs) = FileSystem::from_disk(Disk::secondary_slave(base)) {
            return Some(box FileScheme {
                pci: pci,
                fs: fs,
            });
        }

        None
    }
}

impl SessionItem for FileScheme {
    fn on_irq(&mut self, irq: u8) {
        if irq == self.fs.disk.irq {
            self.on_poll();
        }
    }

    fn on_poll(&mut self) {
        unsafe {
            self.fs.disk.on_poll();
        }
    }

    fn scheme(&self) -> String {
        return "file".to_string();
    }

    fn open(&mut self, url: &URL) -> Option<Box<Resource>> {
        let path = url.path();
        if path.len() == 0 || path.ends_with("/".to_string()) {
            let mut list = String::new();
            let mut dirs: Vec<String> = Vec::new();

            for file in self.fs.list(&path).iter() {
                let line;
                match file.find("/".to_string()) {
                    Some(index) => {
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
                    None => line = file.clone(),
                }
                if line.len() > 0 {
                    if list.len() > 0 {
                        list = list + '\n' + line;
                    } else {
                        list = line;
                    }
                }
            }

            return Some(box VecResource::new(url.clone(), list.to_utf8()));
        } else {
            match self.fs.node(&path) {
                Some(node) => {
                    let mut vec: Vec<u8> = Vec::new();
                    //TODO: Handle more extents
                    for extent in &node.extents {
                        if extent.block > 0 && extent.length > 0 {
                            if let Some(mut data) = Memory::<u8>::new(extent.length as usize) {
                                let sectors = (extent.length as usize + 511) / 512;
                                let mut sector: usize = 0;
                                while sectors - sector >= 65536 {
                                    let request = Request {
                                        extent: Extent {
                                            block: extent.block + sector as u64,
                                            length: 65536 * 512,
                                        },
                                        mem: unsafe { data.address() } + sector * 512,
                                        read: true,
                                        complete: Arc::new(AtomicBool::new(false)),
                                    };

                                    self.fs.disk.request(request.clone());

                                    while !request.complete.load(Ordering::SeqCst) {
                                        unsafe { context_switch(false) };
                                    }

                                    sector += 65535;
                                }
                                if sector < sectors {
                                    let request = Request {
                                        extent: Extent {
                                            block: extent.block + sector as u64,
                                            length: (sectors - sector) as u64 * 512,
                                        },
                                        mem: unsafe { data.address() } + sector * 512,
                                        read: true,
                                        complete: Arc::new(AtomicBool::new(false)),
                                    };

                                    self.fs.disk.request(request.clone());

                                    while !request.complete.load(Ordering::SeqCst) {
                                        unsafe { context_switch(false) };
                                    }
                                }

                                vec.push_all(&Vec {
                                    data: unsafe { data.into_raw() },
                                    length: extent.length as usize,
                                });
                            }
                        }
                    }

                    return Some(box FileResource {
                        scheme: self,
                        node: node,
                        vec: vec,
                        seek: 0,
                        dirty: false,
                    });
                }
                None => return None
            }
        }
    }
}
