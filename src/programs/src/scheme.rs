use alloc::boxed::Box;

use core::ptr;

use common::context::*;
use common::resource::{Resource, ResourceSeek, ResourceType, NoneResource};
use common::elf::*;
use common::memory;
use common::paging::Page;
use common::resource::URL;
use common::scheduler::{start_no_ints, end_no_ints};
use common::string::*;
use common::vec::Vec;

use programs::session::SessionItem;

pub struct SchemeResource {
    handle: usize,
    memory: ContextMemory,
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

    unsafe fn map(&mut self) {
        for i in 0..(self.memory.virtual_size + 4095) / 4096 {
            Page::new(self.memory.virtual_address + i * 4096).map(self.memory.physical_address + i * 4096);
        }
    }

    unsafe fn unmap(&mut self) {
        for i in 0..(self.memory.virtual_size + 4095) / 4096 {
            Page::new(self.memory.virtual_address + i * 4096).map_identity();
        }
    }
}

impl Resource for SchemeResource {
    /// Return the url of this resource
    //TODO
    fn url(&self) -> URL {
        URL::new()
    }

    /// Return the type of this resource
    fn stat(&self) -> ResourceType {
        ResourceType::File
    }

    /// Read data to buffer
    fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        if self.valid(self._read) {
            let result;
            unsafe {
                let reenable = start_no_ints();
                self.map();
                let fn_ptr: *const usize = &self._read;
                result = (*(fn_ptr as *const extern "C" fn(usize, *mut u8, usize) -> usize))(self.handle, buf.as_mut_ptr(), buf.len());
                self.unmap();
                end_no_ints(reenable);
            }
            if result != 0xFFFFFFFF {
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
                let reenable = start_no_ints();
                self.map();
                let fn_ptr: *const usize = &self._write;
                result = (*(fn_ptr as *const extern "C" fn(usize, *const u8, usize) -> usize))(self.handle, buf.as_ptr(), buf.len());
                self.unmap();
                end_no_ints(reenable);
            }
            if result != 0xFFFFFFFF {
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
                let reenable = start_no_ints();
                self.map();
                let fn_ptr: *const usize = &self._lseek;
                result = (*(fn_ptr as *const extern "C" fn(usize, isize, isize) -> usize))(self.handle, offset, whence);
                self.unmap();
                end_no_ints(reenable);
            }
            if result != 0xFFFFFFFF {
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
                let reenable = start_no_ints();
                self.map();
                let fn_ptr: *const usize = &self._fsync;
                result = (*(fn_ptr as *const extern "C" fn(usize) -> usize))(self.handle);
                self.unmap();
                end_no_ints(reenable);
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
                let reenable = start_no_ints();
                self.map();
                let fn_ptr: *const usize = &self._close;
                (*(fn_ptr as *const extern "C" fn(usize) -> usize))(self.handle);
                self.unmap();
                end_no_ints(reenable);
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
            _read: 0,
            _write: 0,
            _lseek: 0,
            _fsync: 0,
            _close: 0,
        };

        {
            let mut resource = url.open();

            let mut vec: Vec<u8> = Vec::new();
            resource.read_to_end(&mut vec);

            unsafe {
                let executable = ELF::from_data(vec.as_ptr() as usize);
                if executable.data > 0 {
                    scheme_item.memory.virtual_size = memory::alloc_size(executable.data) - 4096;
                    scheme_item.memory.physical_address = memory::alloc(scheme_item.memory.virtual_size);
                    ptr::copy((executable.data + 4096) as *const u8,
                              scheme_item.memory.physical_address as *mut u8,
                              scheme_item.memory.virtual_size);

                    scheme_item._start = executable.symbol("_start".to_string());
                    scheme_item._stop = executable.symbol("_stop".to_string());
                    scheme_item._open = executable.symbol("_open".to_string());
                    scheme_item._read = executable.symbol("_read".to_string());
                    scheme_item._write = executable.symbol("_write".to_string());
                    scheme_item._lseek = executable.symbol("_lseek".to_string());
                    scheme_item._fsync = executable.symbol("_fsync".to_string());
                    scheme_item._close = executable.symbol("_close".to_string());
                }
            }
        }

        if scheme_item.valid(scheme_item._start) {
            //TODO: Allow schemes to be called inside of other schemes
            unsafe {
                let reenable = start_no_ints();
                scheme_item.map();
                let fn_ptr: *const usize = &scheme_item._start;
                scheme_item.handle = (*(fn_ptr as *const extern "C" fn() -> usize))();
                scheme_item.unmap();
                end_no_ints(reenable);
            }
        }

        scheme_item
    }

    fn valid(&self, addr: usize) -> bool {
        addr >= self.memory.virtual_address && addr < self.memory.virtual_address + self.memory.virtual_size
    }

    unsafe fn map(&mut self) {
        for i in 0..(self.memory.virtual_size + 4095) / 4096 {
            Page::new(self.memory.virtual_address + i * 4096).map(self.memory.physical_address + i * 4096);
        }
    }

    unsafe fn unmap(&mut self) {
        for i in 0..(self.memory.virtual_size + 4095) / 4096 {
            Page::new(self.memory.virtual_address + i * 4096).map_identity();
        }
    }
}

impl SessionItem for SchemeItem {
    fn scheme(&self) -> String {
        return self.scheme.clone();
    }

    fn open(&mut self, url: &URL) -> Box<Resource> {
        if self.valid(self._open) {
            let fd;
            unsafe {
                let c_str = url.to_string().to_c_str();

                let reenable = start_no_ints();
                self.map();
                let fn_ptr: *const usize = &self._open;
                fd = (*(fn_ptr as *const extern "C" fn(usize, *const u8) -> usize))(self.handle, c_str);
                self.unmap();
                end_no_ints(reenable);

                memory::unalloc(c_str as usize);
            }
            if fd != 0xFFFFFFFF {
                //TODO: Count number of handles, don't allow drop until 0
                return box SchemeResource {
                    handle: fd,
                    memory: ContextMemory {
                        physical_address: self.memory.physical_address,
                        virtual_address: self.memory.virtual_address,
                        virtual_size: self.memory.virtual_size,
                    },
                    _read: self._read,
                    _write: self._write,
                    _lseek: self._lseek,
                    _fsync: self._fsync,
                    _close: self._close,
                };
            }
        }

        box NoneResource
    }
}

impl Drop for SchemeItem {
    fn drop(&mut self) {
        if self.valid(self._stop) {
            unsafe {
                let reenable = start_no_ints();
                self.map();
                let fn_ptr: *const usize = &self._stop;
                (*(fn_ptr as *const extern "C" fn(usize) -> usize))(self.handle);
                self.unmap();
                end_no_ints(reenable);
            }
        }
    }
}
