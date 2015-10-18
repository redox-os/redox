use alloc::boxed::Box;

use core::ptr;
use core::usize;

use common::context::*;
use common::resource::{Resource, ResourceSeek};
use common::elf::*;
use common::memory;
use common::paging::Page;
use common::resource::URL;
use common::scheduler::{start_no_ints, end_no_ints};
use common::string::*;
use common::vec::Vec;

use programs::session::SessionItem;

pub struct SchemeContext {
    interrupts: bool,
    old_memory: Vec<ContextMemory>,
}

impl SchemeContext {
    pub unsafe fn enter(memory: &ContextMemory) -> SchemeContext {
        let interrupts = start_no_ints();
        let mut old_memory: Vec<ContextMemory> = Vec::new();
        for i in 0..(memory.virtual_size + 4095) / 4096 {
            let mut page = Page::new(memory.virtual_address + i * 4096);
            //TODO: Use one contextmemory if possible
            old_memory.push(ContextMemory {
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
            if (ptr as usize) >= memory.virtual_address && (ptr as usize) < memory.virtual_address + memory.virtual_size {
                return ((ptr as usize) - memory.virtual_address + memory.physical_address) as *const T;
            }
        }

        ptr
    }

    pub fn translate_mut<T>(&self, ptr: *mut T) -> *mut T {
        for memory in self.old_memory.iter() {
            if (ptr as usize) >= memory.virtual_address && (ptr as usize) < memory.virtual_address + memory.virtual_size {
                return ((ptr as usize) - memory.virtual_address + memory.physical_address) as *mut T;
            }
        }

        ptr
    }

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

pub struct SchemeResource {
    handle: usize,
    memory: ContextMemory,
    _dup: usize,
    _fpath: usize,
    _read: usize,
    _write: usize,
    _lseek: usize,
    _fsync: usize,
    _close: usize,
}

impl SchemeResource {
    fn valid(&self, addr: usize) -> bool {
        addr >= self.memory.virtual_address && addr < self.memory.virtual_address + self.memory.virtual_size
    }
}

impl Resource for SchemeResource {
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
                //TODO: Count number of handles, don't allow drop until 0
                return Some(box SchemeResource {
                    handle: fd,
                    memory: ContextMemory {
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
                    _close: self._close,
                });
            }
        }

        None
    }

    /// Return the url of this resource
    fn url(&self) -> URL {
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
                return URL::from_string(&String::from_c_slice(&buf));
            }
        }
        URL::new()
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
                },
                ResourceSeek::Current(off) => {
                    whence = 1;
                    offset = off;
                },
                ResourceSeek::End(off) => {
                    whence = 2;
                    offset = off;
                }
            }

            let result;
            unsafe {
                let context = SchemeContext::enter(&self.memory);
                let fn_ptr: *const usize = &self._lseek;
                result = (*(fn_ptr as *const extern "C" fn(usize, isize, isize) -> usize))(self.handle, offset, whence);
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
}

impl Drop for SchemeResource {
    fn drop(&mut self){
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

pub struct SchemeItem {
    scheme: String,
    handle: usize,
    memory: ContextMemory,
    _start: usize,
    _stop: usize,
    _open: usize,
    _dup: usize,
    _fpath: usize,
    _read: usize,
    _write: usize,
    _lseek: usize,
    _fsync: usize,
    _close: usize,
}

impl SchemeItem {
    pub fn from_url(scheme: &String, url: &URL) -> Box<SchemeItem> {
        let mut scheme_item = box SchemeItem {
            scheme: scheme.clone(),
            handle: 0,
            memory: ContextMemory {
                physical_address: 0,
                virtual_address: 0xC0000000,
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
            _close: 0,
        };

        if let Some(mut resource) = url.open() {
            let mut vec: Vec<u8> = Vec::new();
            resource.read_to_end(&mut vec);

            unsafe {
                let executable = ELF::from_data(vec.as_ptr() as usize);
                if executable.data > 0 {
                    scheme_item.memory.virtual_size = memory::alloc_size(executable.data) - ELF_OFFSET;
                    scheme_item.memory.physical_address = memory::alloc(scheme_item.memory.virtual_size);
                    ptr::copy((executable.data + ELF_OFFSET) as *const u8,
                              scheme_item.memory.physical_address as *mut u8,
                              scheme_item.memory.virtual_size);

                    scheme_item._start = executable.symbol("_start");
                    scheme_item._stop = executable.symbol("_stop");
                    scheme_item._open = executable.symbol("_open");
                    scheme_item._dup = executable.symbol("_dup");
                    scheme_item._fpath = executable.symbol("_fpath");
                    scheme_item._read = executable.symbol("_read");
                    scheme_item._write = executable.symbol("_write");
                    scheme_item._lseek = executable.symbol("_lseek");
                    scheme_item._fsync = executable.symbol("_fsync");
                    scheme_item._close = executable.symbol("_close");
                }
            }
        }

        if scheme_item.valid(scheme_item._start) {
            //TODO: Allow schemes to be called inside of other schemes
            unsafe {
                let context = SchemeContext::enter(&scheme_item.memory);
                let fn_ptr: *const usize = &scheme_item._start;
                scheme_item.handle = (*(fn_ptr as *const extern "C" fn() -> usize))();
                context.exit();
            }
        }

        scheme_item
    }

    fn valid(&self, addr: usize) -> bool {
        addr >= self.memory.virtual_address && addr < self.memory.virtual_address + self.memory.virtual_size
    }
}

impl SessionItem for SchemeItem {
    fn scheme(&self) -> String {
        return self.scheme.clone();
    }

    fn open(&mut self, url: &URL) -> Option<Box<Resource>> {
        if self.valid(self._open) {
            let fd;
            unsafe {
                let c_str = url.to_string().to_c_str();

                let context = SchemeContext::enter(&self.memory);
                let fn_ptr: *const usize = &self._open;
                fd = (*(fn_ptr as *const extern "C" fn(usize, *const u8) -> usize))(self.handle, context.translate(c_str));
                context.exit();

                memory::unalloc(c_str as usize);
            }
            if fd != usize::MAX {
                //TODO: Count number of handles, don't allow drop until 0
                return Some(box SchemeResource {
                    handle: fd,
                    memory: ContextMemory {
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
