use alloc::arc::Arc;
use alloc::boxed::Box;

use core::{cmp, ptr};
use core::sync::atomic::{AtomicBool, Ordering};

use drivers::disk::*;
use drivers::pio::*;
use drivers::pciconfig::PCIConfig;

use common::debug;
use common::queue::Queue;
use common::memory;
use common::memory::Memory;
use common::resource::{NoneResource, Resource, ResourceSeek, ResourceType, URL, VecResource};
use common::scheduler::*;
use common::string::{String, ToString};
use common::vec::Vec;

use programs::common::SessionItem;

use syscall::call::sys_yield;

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
    pub fn new(address: u64, data: NodeData) -> Self {
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
    fn clone(&self) -> Self {
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
    pub fn from_disk(disk: Disk) -> Option<Self> {
        unsafe {
            if disk.identify() {
                debug::d(" Disk Found");

                let header_ptr: *const Header = memory::alloc_type();
                disk.read(1, 1, header_ptr as usize);
                let header = ptr::read(header_ptr);
                memory::unalloc(header_ptr as usize);

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
                        let node_data: *const NodeData = memory::alloc_type();
                        for extent in &header.extents {
                            if extent.block > 0 && extent.length > 0 {
                                for node_address in extent.block..extent.block + (extent.length + 511) / 512 {
                                    disk.read(node_address, 1, node_data as usize);

                                    nodes.push(Node::new(node_address, ptr::read(node_data)));
                                }
                            }
                        }
                        memory::unalloc(node_data as usize);

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

        Option::None
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
    pub scheme: *mut FileScheme,
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

                        let reenable = start_no_ints();

                        let mut sector: usize = 0;
                        while sectors - sector >= 65536 {
                            (*self.scheme).fs.disk.write(extent.block + sector as u64,
                                            65535,
                                            data + sector * 512);
                            sector += 65535;
                        }
                        if sector < sectors {
                            (*self.scheme).fs.disk.write(extent.block + sector as u64,
                                            (sectors - sector) as u16,
                                            data + sector * 512);
                        }

                        end_no_ints(reenable);
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

/// A disk request
pub struct Request {
    /// The disk extent
    extent: Extent,
    /// The memory location
    mem: usize,
    /// The request type
    read: bool,
    /// Completion indicator
    complete: Arc<AtomicBool>,
}

impl Clone for Request {
    fn clone(&self) -> Self {
        Request {
            extent: self.extent,
            mem: self.mem,
            read: self.read,
            complete: self.complete.clone(),
        }
    }
}

/// Direction of DMA, set if moving from disk to memory, not set if moving from memory to disk
const CMD_DIR: u8 = 8;
/// DMA should process PRDT
const CMD_ACT: u8 = 1;
/// DMA interrupt occured
const STS_INT: u8 = 4;
/// DMA error occured
const STS_ERR: u8 = 2;
/// DMA is processing PRDT
const STS_ACT: u8 = 1;

/// PRDT End of Table
const PRD_EOT: u32 = 0x80000000;

/// Physical Region Descriptor
#[repr(packed)]
struct PRD {
    addr: u32,
    size: u32,
}

struct PRDT {
    reg: PIO32,
    mem: Memory<PRD>,
}

impl PRDT {
    fn new(port: u16) -> Option<Self> {
        if let Some(mem) = Memory::new_align(8192, 65536) {
            return Some(PRDT {
                reg: PIO32::new(port),
                mem: mem,
            });
        }

        None
    }
}

impl Drop for PRDT {
    fn drop(&mut self) {
        unsafe { self.reg.write(0) };
    }
}

pub struct FileScheme {
    pci: PCIConfig,
    fs: FileSystem,
    request: Option<Request>,
    requests: Queue<Request>,
    cmd: PIO8,
    sts: PIO8,
    prdt: Option<PRDT>,
    irq: u8,
}

impl FileScheme {
    ///TODO Allow busmaster for secondary
    pub fn new(mut pci: PCIConfig) -> Option<Box<Self>> {
        let base = unsafe { pci.read(0x20) } as u16 & 0xFFF0;

        debug::d("IDE on ");
        debug::dh(base as usize);
        debug::dl();

        debug::d("Primary Master:");
        if let Some(fs) = FileSystem::from_disk(Disk::primary_master()) {
            return Some(box FileScheme {
                pci: pci,
                fs: fs,
                request: Option::None,
                requests: Queue::new(),
                cmd: PIO8::new(base),
                sts: PIO8::new(base + 2),
                prdt: PRDT::new(base + 4),
                irq: 0xE,
            });
        }

        debug::d("Primary Slave:");
        if let Some(fs) = FileSystem::from_disk(Disk::primary_slave()) {
            return Some(box FileScheme {
                pci: pci,
                fs: fs,
                request: Option::None,
                requests: Queue::new(),
                cmd: PIO8::new(base),
                sts: PIO8::new(base + 2),
                prdt: PRDT::new(base + 4),
                irq: 0xE,
            });
        }

        debug::d("Secondary Master:");
        if let Some(fs) = FileSystem::from_disk(Disk::secondary_master()) {
            return Some(box FileScheme {
                pci: pci,
                fs: fs,
                request: Option::None,
                requests: Queue::new(),
                cmd: PIO8::new(base + 8),
                sts: PIO8::new(base + 0xA),
                prdt: PRDT::new(base + 0xC),
                irq: 0xF,
            });
        }

        debug::d("Secondary Slave:");
        if let Some(fs) = FileSystem::from_disk(Disk::secondary_slave()) {
            return Some(box FileScheme {
                pci: pci,
                fs: fs,
                request: Option::None,
                requests: Queue::new(),
                cmd: PIO8::new(base + 8),
                sts: PIO8::new(base + 0xA),
                prdt: PRDT::new(base + 0xC),
                irq: 0xF,
            });
        }

        None
    }

    pub fn request(&mut self, request: Request) {
        unsafe {
            let reenable = start_no_ints();

            self.requests.push(request);

            if self.request.is_none() {
                self.next_request();
            }

            end_no_ints(reenable);
        }
    }

    unsafe fn next_request(&mut self) {
        let reenable = start_no_ints();

        self.cmd.write(CMD_DIR);
        if let Some(ref mut prdt) = self.prdt {
            prdt.reg.write(0 as u32);
        }

        if let Some(ref mut req) = self.request {
            req.complete.store(true, Ordering::SeqCst);
        }

        self.request = self.requests.pop();

        if let Some(ref mut req) = self.request {
            req.complete.store(false, Ordering::SeqCst);

            if req.mem > 0 {
                if let Some(ref mut prdt) = self.prdt {
                    let sectors = (req.extent.length + 511)/512;
                    let mut size = sectors * 512;
                    let mut i = 0;
                    while size >= 65536 && i < 8192 {
                        let eot;
                        if size == 65536 {
                            eot = PRD_EOT;
                        } else {
                            eot = 0;
                        }

                        unsafe{
                            prdt.mem.write(i, PRD {
                               addr: (req.mem + i * 65536) as u32,
                               size: eot,
                            });
                        }

                        size -= 65536;
                        i += 1;
                    }
                    if size > 0 && i < 8192 {
                        unsafe {
                            prdt.mem.write(i, PRD {
                               addr: (req.mem + i * 65536) as u32,
                               size: size as u32 | PRD_EOT,
                            });
                        }

                        size = 0;
                        i += 1;
                    }

                    if i > 0 {
                        if size == 0 {
                            if req.read {
                                prdt.reg.write(prdt.mem.ptr as u32);
                                //self.fs.disk.read(req.extent.block, sectors as u16, req.mem);
                                self.fs.disk.read_dma(req.extent.block, sectors);
                                self.cmd.write(CMD_ACT | CMD_DIR);
                            } else {
                                prdt.reg.write(prdt.mem.ptr as u32);
                                self.fs.disk.write_dma(req.extent.block, sectors);
                                self.cmd.write(CMD_ACT);
                            }
                        }else{
                            debug::d("IDE Request too large: ");
                            debug::dd(size as usize);
                            debug::d(" remaining\n");
                        }
                    }else{
                        debug::d("IDE Request size is 0\n");
                    }
                }else{
                    debug::d("PRDT not allocated\n");
                }
            }else{
                debug::d("IDE Request mem is 0\n");
            }
        }

        end_no_ints(reenable);
    }
}

impl SessionItem for FileScheme {
    fn on_irq(&mut self, irq: u8) {
        if irq == self.irq {
            self.on_poll();
        }
    }

    fn on_poll(&mut self) {
        let sts = unsafe { self.sts.read() };
        if sts & STS_INT == STS_INT {
            unsafe { self.sts.write(sts) };

            let cmd = unsafe { self.cmd.read() };
            if cmd & CMD_ACT == CMD_ACT {
                if cmd & CMD_DIR == CMD_DIR {
                    debug::d("IDE DMA READ\n");
                } else {
                    debug::d("IDE DMA WRITE\n");
                }

                unsafe { self.next_request() };
            } else {
                debug::d("IDE PIO\n");

                unsafe { self.next_request() };
            }
        }
    }

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
                                    let sectors = (extent.length as usize + 511) / 512;
                                    let mut sector: usize = 0;
                                    while sectors - sector >= 65536 {
                                        let request = Request {
                                            extent: Extent {
                                                block: extent.block + sector as u64,
                                                length: 65536 * 512,
                                            },
                                            mem: data + sector * 512,
                                            read: true,
                                            complete: Arc::new(AtomicBool::new(false)),
                                        };

                                        path.d();
                                        debug::dl();

                                        self.request(request.clone());

                                        while request.complete.load(Ordering::SeqCst) == false {
                                            sys_yield();
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
                                            read: true,
                                            complete: Arc::new(AtomicBool::new(false)),
                                        };

                                        path.d();
                                        debug::dl();

                                        self.request(request.clone());

                                        while request.complete.load(Ordering::SeqCst) == false {
                                            sys_yield();
                                        }
                                    }

                                    vec.push_all(&Vec {
                                        data: data as *mut u8,
                                        length: extent.length as usize,
                                    });
                                }
                            }
                        }
                    }

                    return box FileResource {
                        scheme: self,
                        node: node,
                        vec: vec,
                        seek: 0,
                        dirty: false,
                    };
                }
                Option::None => {
                    return box NoneResource;
                }
            }
        }
    }
}
