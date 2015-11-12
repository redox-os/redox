use common::event::Event;
use common::get_slice::GetSlice;

use alloc::boxed::Box;

use collections::string::{String, ToString};
use collections::vec::Vec;

use core::usize;

use common::debug;
use common::elf::Elf;
use common::memory;
use common::paging::Page;

use scheduler::{start_no_ints, end_no_ints};

use schemes::{KScheme, Resource, ResourceSeek, Url};

pub struct SchemeMemory {
    pub physical_address: usize,
    pub virtual_address: usize,
    pub virtual_size: usize,
}

impl SchemeMemory {
    pub unsafe fn map(&mut self) {
        for i in 0..(self.virtual_size + 4095) / 4096 {
            Page::new(self.virtual_address + i * 4096).map(self.physical_address + i * 4096);
        }
    }
    pub unsafe fn unmap(&mut self) {
        for i in 0..(self.virtual_size + 4095) / 4096 {
            Page::new(self.virtual_address + i * 4096).map_identity();
        }
    }
}

/// A scheme context
pub struct SchemeContext {
    /// Interrupted
    interrupts: bool,
    /// The old memory (before context switch)
    old_memory: Vec<SchemeMemory>,
}

impl SchemeContext {
    /// Enter from a given context memory
    pub unsafe fn enter(memory: &SchemeMemory) -> SchemeContext {
        let interrupts = start_no_ints();
        let mut old_memory: Vec<SchemeMemory> = Vec::new();
        for i in 0..(memory.virtual_size + 4095) / 4096 {
            let mut page = Page::new(memory.virtual_address + i * 4096);
            // TODO: Use one SchemeMemory if possible
            old_memory.push(SchemeMemory {
                physical_address: page.phys_addr(),
                virtual_address: page.virt_addr(),
                virtual_size: 4096,
            });
            page.map(memory.physical_address + i * 4096);
        }

        SchemeContext {
            interrupts: interrupts,
            old_memory: old_memory,
        }
    }

    pub fn translate<T>(&self, ptr: *const T) -> *const T {
        for memory in self.old_memory.iter() {
            if (ptr as usize) >= memory.virtual_address &&
               (ptr as usize) < memory.virtual_address + memory.virtual_size {
                return ((ptr as usize) - memory.virtual_address +
                        memory.physical_address) as *const T;
            }
        }

        ptr
    }

    pub fn translate_mut<T>(&self, ptr: *mut T) -> *mut T {
        for memory in self.old_memory.iter() {
            if (ptr as usize) >= memory.virtual_address &&
               (ptr as usize) < memory.virtual_address + memory.virtual_size {
                return ((ptr as usize) - memory.virtual_address +
                        memory.physical_address) as *mut T;
            }
        }

        ptr
    }

    /// Exit the context
    pub unsafe fn exit(self) {
        for memory in self.old_memory.iter() {
            for i in 0..(memory.virtual_size + 4095) / 4096 {
                let mut page = Page::new(memory.virtual_address + i * 4096);
                page.map(memory.physical_address + i * 4096);
            }
        }
        end_no_ints(self.interrupts);
    }
}

/// A scheme resource
pub struct SchemeResource {
    /// The handle
    handle: usize,
    /// The scheme memory
    memory: SchemeMemory,
    /// Duplicate?
    _dup: usize,
    /// Internal fpath
    _fpath: usize,
    /// Internal read
    _read: usize,
    /// Internal write
    _write: usize,
    /// Internal lseek
    _lseek: usize,
    /// Internal fsync
    _fsync: usize,
    /// Internal ftruncate
    _ftruncate: usize,
    /// Internal close
    _close: usize,
}

impl SchemeResource {
    /// Check validity
    fn valid(&self, addr: usize) -> bool {
        addr >= self.memory.virtual_address &&
        addr < self.memory.virtual_address + self.memory.virtual_size
    }
}

impl Resource for SchemeResource {
    // TODO: Clone instead?
    /// Duplicate the resource
    fn dup(&self) -> Option<Box<Resource>> {
        if self.valid(self._dup) {
            let fd;
            unsafe {
                let context = SchemeContext::enter(&self.memory);
                let fn_ptr: *const usize = &self._dup;
                fd = (*(fn_ptr as *const extern "C" fn(usize) -> usize))(self.handle);
                context.exit();
            }
            if fd != usize::MAX {
                // TODO: Count number of handles, don't allow drop until 0
                return Some(box SchemeResource {
                    handle: fd,
                    memory: SchemeMemory {
                        physical_address: self.memory.physical_address,
                        virtual_address: self.memory.virtual_address,
                        virtual_size: self.memory.virtual_size,
                    },
                    _dup: self._dup,
                    _fpath: self._fpath,
                    _read: self._read,
                    _write: self._write,
                    _lseek: self._lseek,
                    _fsync: self._fsync,
                    _ftruncate: self._ftruncate,
                    _close: self._close,
                });
            }
        }

        None
    }

    /// Return the url of this resource
    fn url(&self) -> Url {
        if self.valid(self._fpath) {
            let mut buf: [u8; 4096] = [0; 4096];
            let result;
            unsafe {
                let context = SchemeContext::enter(&self.memory);
                let fn_ptr: *const usize = &self._fpath;
                result = (*(fn_ptr as *const extern "C" fn(usize, *mut u8, usize) -> usize))(self.handle, context.translate_mut(buf.as_mut_ptr()), buf.len());
                context.exit();
            }
            if result != usize::MAX {
                return Url::from_string(unsafe {
                    String::from_utf8_unchecked(Vec::from(buf.get_slice(None, Some(result))))
                });
            }
        }
        Url::new()
    }

    /// Read data to buffer
    fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        if self.valid(self._read) {
            let result;
            unsafe {
                let context = SchemeContext::enter(&self.memory);
                let fn_ptr: *const usize = &self._read;
                result = (*(fn_ptr as *const extern "C" fn(usize, *mut u8, usize) -> usize))(self.handle, context.translate_mut(buf.as_mut_ptr()), buf.len());
                context.exit();
            }
            if result != usize::MAX {
                return Some(result);
            }
        }
        None
    }

    /// Write to resource
    fn write(&mut self, buf: &[u8]) -> Option<usize> {
        if self.valid(self._write) {
            let result;
            unsafe {
                let context = SchemeContext::enter(&self.memory);
                let fn_ptr: *const usize = &self._write;
                result = (*(fn_ptr as *const extern "C" fn(usize, *const u8, usize) -> usize))(self.handle, context.translate(buf.as_ptr()), buf.len());
                context.exit();
            }
            if result != usize::MAX {
                return Some(result);
            }
        }
        None
    }

    /// Seek
    fn seek(&mut self, pos: ResourceSeek) -> Option<usize> {
        if self.valid(self._lseek) {
            let offset;
            let whence;
            match pos {
                ResourceSeek::Start(off) => {
                    whence = 0;
                    offset = off as isize;
                }
                ResourceSeek::Current(off) => {
                    whence = 1;
                    offset = off;
                }
                ResourceSeek::End(off) => {
                    whence = 2;
                    offset = off;
                }
            }

            let result;
            unsafe {
                let context = SchemeContext::enter(&self.memory);
                let fn_ptr: *const usize = &self._lseek;
                result =
                    (*(fn_ptr as *const extern "C" fn(usize, isize, isize) -> usize))(self.handle,
                                                                                      offset,
                                                                                      whence);
                context.exit();
            }
            if result != usize::MAX {
                return Some(result);
            }
        }
        None
    }

    /// Sync the resource
    fn sync(&mut self) -> bool {
        if self.valid(self._fsync) {
            let result;
            unsafe {
                let context = SchemeContext::enter(&self.memory);
                let fn_ptr: *const usize = &self._fsync;
                result = (*(fn_ptr as *const extern "C" fn(usize) -> usize))(self.handle);
                context.exit();
            }
            return result == 0;
        }
        false
    }

    fn truncate(&mut self, len: usize) -> bool {
        if self.valid(self._ftruncate) {
            let result;
            unsafe {
                let context = SchemeContext::enter(&self.memory);
                let fn_ptr: *const usize = &self._ftruncate;
                result = (*(fn_ptr as *const extern "C" fn(usize, usize) -> usize))(self.handle,
                                                                                    len);
                context.exit();
            }
            return result == 0;
        }
        false
    }
}

impl Drop for SchemeResource {
    fn drop(&mut self) {
        if self.valid(self._close) {
            unsafe {
                let context = SchemeContext::enter(&self.memory);
                let fn_ptr: *const usize = &self._close;
                (*(fn_ptr as *const extern "C" fn(usize) -> usize))(self.handle);
                context.exit();
            }
        }
    }
}

/// A scheme item
pub struct SchemeItem {
    /// The URL
    url: Url,
    /// The scheme
    scheme: String,
    /// The binary for the scheme
    binary: Url,
    /// The handle
    handle: usize,
    /// The scheme memory
    memory: SchemeMemory,
    _start: usize,
    _stop: usize,
    _open: usize,
    _dup: usize,
    _fpath: usize,
    _read: usize,
    _write: usize,
    _lseek: usize,
    _fsync: usize,
    _ftruncate: usize,
    _close: usize,
    _event: usize,
}

impl SchemeItem {
    /// Load scheme item from URL
    pub fn from_url(url: &Url) -> Box<SchemeItem> {
        let mut scheme_item = box SchemeItem {
            url: url.clone(),
            scheme: String::new(),
            binary: Url::from_string(url.to_string() + "main.bin"),
            handle: 0,
            memory: SchemeMemory {
                physical_address: 0,
                virtual_address: 0,
                virtual_size: 0,
            },
            _start: 0,
            _stop: 0,
            _open: 0,
            _dup: 0,
            _fpath: 0,
            _read: 0,
            _write: 0,
            _lseek: 0,
            _fsync: 0,
            _ftruncate: 0,
            _close: 0,
            _event: 0,
        };

        for part in url.reference().rsplit('/') {
            if ! part.is_empty() {
                scheme_item.scheme = part.to_string();
                break;
            }
        }

        if let Some(mut resource) = scheme_item.binary.open() {
            let mut vec: Vec<u8> = Vec::new();
            resource.read_to_end(&mut vec);

            unsafe {
                let executable = Elf::from_data(vec.as_ptr() as usize);
                if let Some(segment) = executable.load_segment() {
                    scheme_item.memory.virtual_address = segment.vaddr as usize;
                    scheme_item.memory.virtual_size = segment.mem_len as usize;
                    scheme_item.memory.physical_address = memory::alloc(scheme_item.memory
                                                                                   .virtual_size);

                    if scheme_item.memory.physical_address > 0 {
                        // Copy progbits
                        ::memcpy(scheme_item.memory.physical_address as *mut u8,
                                 (executable.data + segment.off as usize) as *const u8,
                                 segment.file_len as usize);
                        // Zero bss
                        ::memset((scheme_item.memory.physical_address + segment.file_len as usize) as *mut u8, 0, segment.mem_len as usize - segment.file_len as usize);
                    }

                    scheme_item._start = executable.symbol("_start");
                    scheme_item._stop = executable.symbol("_stop");
                    scheme_item._open = executable.symbol("_open");
                    scheme_item._dup = executable.symbol("_dup");
                    scheme_item._fpath = executable.symbol("_fpath");
                    scheme_item._read = executable.symbol("_read");
                    scheme_item._write = executable.symbol("_write");
                    scheme_item._lseek = executable.symbol("_lseek");
                    scheme_item._fsync = executable.symbol("_fsync");
                    scheme_item._ftruncate = executable.symbol("_ftruncate");
                    scheme_item._close = executable.symbol("_close");
                    scheme_item._event = executable.symbol("_event");
                } else {
                    debug::d("Invalid ELF\n");
                }
            }
        }

        if scheme_item.valid(scheme_item._start) {
            // TODO: Allow schemes to be called inside of other schemes
            unsafe {
                let context = SchemeContext::enter(&scheme_item.memory);
                let fn_ptr: *const usize = &scheme_item._start;
                scheme_item.handle = (*(fn_ptr as *const extern "C" fn() -> usize))();
                context.exit();
            }
        }

        scheme_item
    }

    /// Check validity
    fn valid(&self, addr: usize) -> bool {
        addr >= self.memory.virtual_address &&
        addr < self.memory.virtual_address + self.memory.virtual_size
    }
}

impl KScheme for SchemeItem {
    fn scheme(&self) -> &str {
        return &self.scheme;
    }

    // TODO: Hack for orbital
    fn event(&mut self, event: &Event) {
        if self.valid(self._event) {
            unsafe {
                let event_ptr: *const Event = event;

                let context = SchemeContext::enter(&self.memory);
                let fn_ptr: *const usize = &self._event;
                (*(fn_ptr as *const extern "C" fn(usize, usize)))(self.handle, event_ptr as usize);
                context.exit();
            }
        }
    }

    fn open(&mut self, url: &Url, flags: usize) -> Option<Box<Resource>> {
        if self.valid(self._open) {
            let fd;
            unsafe {
                let c_str = url.to_string() + "\0";

                let context = SchemeContext::enter(&self.memory);
                let fn_ptr: *const usize = &self._open;
                fd = (*(fn_ptr as *const extern "C" fn(usize, *const u8, usize) -> usize))(self.handle, context.translate(c_str.as_ptr()), flags);
                context.exit();
            }
            if fd != usize::MAX {
                // TODO: Count number of handles, don't allow drop until 0
                return Some(box SchemeResource {
                    handle: fd,
                    memory: SchemeMemory {
                        physical_address: self.memory.physical_address,
                        virtual_address: self.memory.virtual_address,
                        virtual_size: self.memory.virtual_size,
                    },
                    _dup: self._dup,
                    _fpath: self._fpath,
                    _read: self._read,
                    _write: self._write,
                    _lseek: self._lseek,
                    _fsync: self._fsync,
                    _ftruncate: self._ftruncate,
                    _close: self._close,
                });
            }
        }

        None
    }
}

impl Drop for SchemeItem {
    fn drop(&mut self) {
        if self.valid(self._stop) {
            unsafe {
                let context = SchemeContext::enter(&self.memory);
                let fn_ptr: *const usize = &self._stop;
                (*(fn_ptr as *const extern "C" fn(usize) -> usize))(self.handle);
                context.exit();
            }
        }
    }
}
