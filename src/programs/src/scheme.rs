use alloc::boxed::Box;

use core::ptr;

use common::context::*;
use common::elf::*;
use common::memory;
use common::paging::Page;
use common::resource::URL;
use common::scheduler;
use common::string::*;
use common::vec::Vec;

pub struct Scheme {
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

impl Scheme {
    pub fn from_url(url: &URL) -> Box<Scheme> {
        let mut scheme = box Scheme {
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
                    scheme.memory.virtual_size = memory::alloc_size(executable.data) - 4096;
                    scheme.memory.physical_address = memory::alloc(scheme.memory.virtual_size);
                    ptr::copy((executable.data + 4096) as *const u8,
                              scheme.memory.physical_address as *mut u8,
                              scheme.memory.virtual_size);

                    scheme._start = executable.symbol("_start".to_string());
                    scheme._stop = executable.symbol("_stop".to_string());
                    scheme._open = executable.symbol("_open".to_string());
                    scheme._read = executable.symbol("_read".to_string());
                    scheme._write = executable.symbol("_write".to_string());
                    scheme._lseek = executable.symbol("_lseek".to_string());
                    scheme._fsync = executable.symbol("_fsync".to_string());
                    scheme._close = executable.symbol("_close".to_string());
                }
            }
        }

        if scheme.valid(scheme._start) {
            //TODO: Allow schemes to be called inside of other schemes
            unsafe {
                scheme.map();
                let fn_ptr: *const usize = &scheme._start;
                scheme.handle = (*(fn_ptr as *const extern "C" fn() -> usize))();
                scheme.unmap();
            }
        }

        scheme
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

    pub fn open(&mut self, path: &str) -> usize {
        if self.valid(self._open) && self.handle > 0 {
            let ret;
            unsafe {
                self.map();
                let fn_ptr: *const usize = &self._open;
                ret = (*(fn_ptr as *const extern "C" fn(usize, &str) -> usize))(self.handle, path);
                self.unmap();
            }
            ret
        } else {
            0xFFFFFFFF
        }
    }
}

impl Drop for Scheme {
    fn drop(&mut self) {
        if self.valid(self._stop) && self.handle > 0 {
            unsafe {
                self.map();
                let fn_ptr: *const usize = &self._stop;
                (*(fn_ptr as *const extern "C" fn(usize) -> usize))(self.handle);
                self.unmap();
            }
        }
    }
}
