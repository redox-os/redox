use common::get_slice::GetSlice;

use alloc::arc::Arc;
use alloc::boxed::Box;

use collections::slice;
use collections::string::{String, ToString};
use collections::vec::Vec;

use core::{cmp, mem};
use core::sync::atomic::{AtomicBool, Ordering};

use drivers::disk::{Disk, Extent, Request};
use drivers::pciconfig::PciConfig;

use common::debug;
use common::memory::Memory;

use schemes::{Result, KScheme, Resource, ResourceSeek, Url, VecResource};

use scheduler::context::context_switch;

use sync::Intex;

use syscall::{SysError, O_CREAT, ENOENT, EIO};

const PIO: bool = false;

/// The header of the fs
#[repr(packed)]
pub struct Header {
    pub signature: [u8; 8],
    pub version: u64,
    pub free_space: Extent,
    pub padding: [u8; 224],
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
    pub block: u64,
    pub name: String,
    pub extents: [Extent; 16],
}

impl Node {
    /// Create a new file node from an address and some data
    pub fn new(block: u64, data: &NodeData) -> Self {
        let mut bytes = Vec::new();
        for b in data.name.iter() {
            if *b > 0 {
                bytes.push(*b);
            } else {
                break;
            }
        }

        Node {
            block: block,
            name: unsafe { String::from_utf8_unchecked(bytes) },
            extents: data.extents,
        }
    }

    pub fn data(&self) -> NodeData {
        let mut name: [u8; 256] = [0; 256];
        let mut i = 0;
        for b in self.name.as_bytes().iter() {
            if i < name.len() {
                name[i] = *b;
            } else {
                break;
            }
            i += 1;
        }
        NodeData {
            name: name,
            extents: self.extents,
        }
    }
}

impl Clone for Node {
    fn clone(&self) -> Self {
        Node {
            block: self.block,
            name: self.name.clone(),
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
    pub fn from_disk(disk: Disk) -> Option<Self> {
        unsafe {
            if disk.identify() {
                debug::d(" Disk Found");

                let header_ptr = Memory::<Header>::new(1).unwrap();
                disk.read(1, 1, header_ptr.address());
                let header = header_ptr.read(0);
                drop(header_ptr);

                if header.signature[0] == 'R' as u8 && header.signature[1] == 'E' as u8 &&
                   header.signature[2] == 'D' as u8 &&
                   header.signature[3] == 'O' as u8 &&
                   header.signature[4] == 'X' as u8 &&
                   header.signature[5] == 'F' as u8 &&
                   header.signature[6] == 'S' as u8 &&
                   header.signature[7] == '\0' as u8 && header.version == 1 {

                    debug::d(" Redox Filesystem\n");

                    let mut nodes = Vec::new();
                    for extent in &header.extents {
                        if extent.block > 0 && extent.length > 0 {
                            if let Some(data) =
                                   Memory::<NodeData>::new(extent.length as usize /
                                                           mem::size_of::<NodeData>()) {
                                let sectors = (extent.length as usize + 511) / 512;
                                let mut sector: usize = 0;
                                while sectors - sector >= 65536 {
                                    disk.read(extent.block + sector as u64,
                                              0,
                                              data.address() + sector * 512);

                                    //
                                    // let request = Request {
                                    // extent: Extent {
                                    // block: extent.block + sector as u64,
                                    // length: 65536 * 512,
                                    // },
                                    // mem: data.address() + sector * 512,
                                    // read: true,
                                    // complete: Arc::new(AtomicBool::new(false)),
                                    // };
                                    //
                                    // disk.request(request.clone());
                                    //
                                    // while request.complete.load(Ordering::SeqCst) == false {
                                    // disk.on_poll();
                                    // }
                                    //

                                    sector += 65535;
                                }
                                if sector < sectors {
                                    disk.read(extent.block + sector as u64,
                                              (sectors - sector) as u16,
                                              data.address() + sector * 512);
                                    //
                                    // let request = Request {
                                    // extent: Extent {
                                    // block: extent.block + sector as u64,
                                    // length: (sectors - sector) as u64 * 512,
                                    // },
                                    // mem: data.address() + sector * 512,
                                    // read: true,
                                    // complete: Arc::new(AtomicBool::new(false)),
                                    // };
                                    //
                                    // disk.request(request.clone());
                                    //
                                    // while request.complete.load(Ordering::SeqCst) == false {
                                    // disk.on_poll();
                                    // }
                                    //
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
    pub fn node(&self, filename: &str) -> Option<Node> {
        for node in self.nodes.iter() {
            if node.name == filename {
                return Some(node.clone());
            }
        }

        None
    }

    /// List nodes in a given directory
    pub fn list(&self, directory: &str) -> Vec<String> {
        let mut ret = Vec::new();

        for node in self.nodes.iter() {
            if node.name.starts_with(directory) {
                ret.push(node.name.get_slice(Some(directory.len()), None).to_string());
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
    fn dup(&self) -> Result<Box<Resource>> {
        Ok(box FileResource {
            scheme: self.scheme,
            node: self.node.clone(),
            vec: self.vec.clone(),
            seek: self.seek,
            dirty: self.dirty,
        })
    }

    fn url(&self) -> Url {
        Url::from_string("file:/".to_string() + &self.node.name)
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut i = 0;
        while i < buf.len() && self.seek < self.vec.len() {
            match self.vec.get(self.seek) {
                Some(b) => buf[i] = *b,
                None => (),
            }
            self.seek += 1;
            i += 1;
        }
        Ok(i)
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let mut i = 0;
        while i < buf.len() && self.seek < self.vec.len() {
            self.vec[self.seek] = buf[i];
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
        Ok(i)
    }

    fn seek(&mut self, pos: ResourceSeek) -> Result<usize> {
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
        Ok(self.seek)
    }

    // TODO: Rename to sync
    // TODO: Check to make sure proper amount of bytes written. See Disk::write
    // TODO: Allow reallocation
    fn sync(&mut self) -> Result<()> {
        if self.dirty {
            let block_size: usize = 512;

            let mut node_dirty = false;
            let mut pos: isize = 0;
            let mut remaining = self.vec.len() as isize;
            for ref mut extent in &mut self.node.extents {
                if remaining > 0 && extent.empty() {
                    debug::d("Reallocate file, extra: ");
                    debug::ds(remaining);
                    debug::dl();

                    unsafe {
                        let _intex = Intex::static_lock();

                        let sectors = ((remaining + 511) / 512) as u64;
                        if (*self.scheme).fs.header.free_space.length >= sectors * 512 {
                            extent.block = (*self.scheme).fs.header.free_space.block;
                            extent.length = remaining as u64;
                            (*self.scheme).fs.header.free_space.block = (*self.scheme)
                                                                            .fs
                                                                            .header
                                                                            .free_space
                                                                            .block +
                                                                        sectors;
                            (*self.scheme).fs.header.free_space.length = (*self.scheme)
                                                                             .fs
                                                                             .header
                                                                             .free_space
                                                                             .length -
                                                                         sectors * 512;

                            node_dirty = true;
                        }
                    }
                }

                // Make sure it is a valid extent
                if !extent.empty() {
                    let current_sectors = (extent.length as usize + block_size - 1) / block_size;
                    let max_size = current_sectors * 512;

                    let size = cmp::min(remaining as usize, max_size);

                    if size as u64 != extent.length {
                        extent.length = size as u64;
                        node_dirty = true;
                    }

                    unsafe {
                        let data = self.vec.as_ptr().offset(pos) as usize;
                        // TODO: Make sure data is copied safely into an zeroed area of the right size!

                        let sectors = (extent.length as usize + 511) / 512;
                        let mut sector: usize = 0;
                        while sectors - sector >= 65536 {
                            if PIO {
                                (*self.scheme)
                                    .fs
                                    .disk
                                    .write(extent.block + sector as u64, 0, data + sector * 512);
                            } else {
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
                            }

                            sector += 65535;
                        }
                        if sector < sectors {
                            if PIO {
                                (*self.scheme).fs.disk.write(extent.block + sector as u64,
                                                             (sectors - sector) as u16,
                                                             data + sector * 512);
                            } else {
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
                    }

                    pos += size as isize;
                    remaining -= size as isize;
                }
            }

            if node_dirty {
                debug::d("Node dirty, rewrite\n");

                if self.node.block > 0 {
                    unsafe {
                        if let Some(mut node_data) = Memory::<NodeData>::new(1) {
                            node_data.write(0, self.node.data());

                            if PIO {
                                (*self.scheme)
                                    .fs
                                    .disk
                                    .write(self.node.block, 1, node_data.address());
                            } else {
                                let request = Request {
                                    extent: Extent {
                                        block: self.node.block,
                                        length: 1,
                                    },
                                    mem: node_data.address(),
                                    read: false,
                                    complete: Arc::new(AtomicBool::new(false)),
                                };

                                debug::d("Disk request\n");

                                (*self.scheme).fs.disk.request(request.clone());

                                debug::d("Wait request\n");
                                while request.complete.load(Ordering::SeqCst) == false {
                                    context_switch(false);
                                }
                            }

                            debug::d("Renode\n");

                            {
                                let _intex = Intex::static_lock();

                                for mut node in (*self.scheme).fs.nodes.iter_mut() {
                                    if node.block == self.node.block {
                                        *node = self.node.clone();
                                    }
                                }
                            }
                        }
                    }
                } else {
                    debug::d("Need to place Node block\n");
                }
            }

            self.dirty = false;

            if remaining > 0 {
                debug::d("Need to defragment file, extra: ");
                debug::ds(remaining);
                debug::dl();
                return Err(SysError::new(EIO));
            }
        }
        Ok(())
    }

    fn truncate(&mut self, len: usize) -> Result<()> {
        while len > self.vec.len() {
            self.vec.push(0);
        }
        self.vec.truncate(len);
        self.seek = cmp::min(self.seek, self.vec.len());
        self.dirty = true;
        Ok(())
    }
}

impl Drop for FileResource {
    fn drop(&mut self) {
        self.sync();
    }
}

/// A file scheme (pci + fs)
pub struct FileScheme {
    pci: PciConfig,
    fs: FileSystem,
}

impl FileScheme {
    ///TODO Allow busmaster for secondary
    /// Create a new file scheme from a PCI configuration
    pub fn new(mut pci: PciConfig) -> Option<Box<Self>> {
        unsafe { pci.flag(4, 4, true) }; // Bus mastering

        let base = unsafe { pci.read(0x20) } as u16 & 0xFFF0;

        debug::d("IDE on ");
        debug::dh(base as usize);
        debug::dl();

        debug::d("Primary Master:");
        if let Some(fs) = FileSystem::from_disk(Disk::primary_master(base)) {
            return Some(box FileScheme { pci: pci, fs: fs });
        }

        debug::d("Primary Slave:");
        if let Some(fs) = FileSystem::from_disk(Disk::primary_slave(base)) {
            return Some(box FileScheme { pci: pci, fs: fs });
        }

        debug::d("Secondary Master:");
        if let Some(fs) = FileSystem::from_disk(Disk::secondary_master(base)) {
            return Some(box FileScheme { pci: pci, fs: fs });
        }

        debug::d("Secondary Slave:");
        if let Some(fs) = FileSystem::from_disk(Disk::secondary_slave(base)) {
            return Some(box FileScheme { pci: pci, fs: fs });
        }

        None
    }
}

impl KScheme for FileScheme {
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

    fn scheme(&self) -> &str {
        "file"
    }

    fn open(&mut self, url: &Url, flags: usize) -> Result<Box<Resource>> {
        let mut path = url.reference();
        while path.starts_with('/') {
            path = &path[1..];
        }
        if path.is_empty() || path.ends_with('/') {
            let mut list = String::new();
            let mut dirs: Vec<String> = Vec::new();

            // Hmm... no deref coercions in libcollections ;(
            for file in self.fs.list(path).iter() {
                let mut line = String::new();
                match file.find('/') {
                    Some(index) => {
                        let dirname = file.get_slice(None, Some(index + 1)).to_string();
                        let mut found = false;
                        for dir in dirs.iter() {
                            if dirname == *dir {
                                found = true;
                                break;
                            }
                        }
                        if found {
                            line.clear();
                        } else {
                            line = dirname.clone();
                            dirs.push(dirname);
                        }
                    }
                    None => line = file.clone(),
                }
                if !line.is_empty() {
                    if !list.is_empty() {
                        list = list + "\n" + &line;
                    } else {
                        list = line;
                    }
                }
            }

            if list.len() > 0 {
                Ok(box VecResource::new(url.clone(), list.into_bytes()))
            } else {
                Err(SysError::new(ENOENT))
            }
        } else {
            match self.fs.node(path) {
                Some(node) => {
                    let mut vec: Vec<u8> = Vec::new();
                    // TODO: Handle more extents
                    for extent in &node.extents {
                        if extent.block > 0 && extent.length > 0 {
                            if let Some(data) = Memory::<u8>::new(extent.length as usize) {
                                let sectors = (extent.length as usize + 511) / 512;
                                let mut sector: usize = 0;
                                while sectors - sector >= 65536 {
                                    if PIO {
                                        unsafe {
                                            self.fs.disk.read(extent.block + sector as u64,
                                                              0,
                                                              data.address() + sector * 512);
                                        }
                                    } else {
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
                                    }

                                    sector += 65535;
                                }
                                if sector < sectors {
                                    if PIO {
                                        unsafe {
                                            self.fs.disk.read(extent.block + sector as u64,
                                                              (sectors - sector) as u16,
                                                              data.address() + sector * 512);
                                        }
                                    } else {
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
                                }

                                vec.push_all(&unsafe {
                                    slice::from_raw_parts(data.ptr, extent.length as usize)
                                });
                            }
                        }
                    }

                    Ok(box FileResource {
                        scheme: self,
                        node: node,
                        vec: vec,
                        seek: 0,
                        dirty: false,
                    })
                }
                None => {
                    if flags & O_CREAT == O_CREAT {
                        // TODO: Create file
                        let mut node = Node {
                            block: 0,
                            name: path.to_string(),
                            extents: [Extent {
                                block: 0,
                                length: 0,
                            }; 16],
                        };

                        if self.fs.header.free_space.length >= 512 {
                            node.block = self.fs.header.free_space.block;
                            self.fs.header.free_space.block = self.fs.header.free_space.block + 1;
                            self.fs.header.free_space.length = self.fs.header.free_space.length -
                                                               512;
                        }

                        self.fs.nodes.push(node.clone());

                        Ok(box FileResource {
                            scheme: self,
                            node: node,
                            vec: Vec::new(),
                            seek: 0,
                            dirty: false,
                        })
                    } else {
                        Err(SysError::new(ENOENT))
                    }
                }
            }
        }
    }

    fn unlink(&mut self, url: &Url) -> Result<()> {
        let mut ret = Err(SysError::new(ENOENT));

        let mut path = url.reference();
        while path.starts_with('/') {
            path = &path[1..];
        }

        let mut i = 0;
        while i < self.fs.nodes.len() {
            let mut remove = false;

            if let Some(node) = self.fs.nodes.get(i) {
                remove = node.name == path;
            }

            if remove {
                self.fs.nodes.remove(i);
                ret = Ok(());
            } else {
                i += 1;
            }
        }

        ret
    }
}
