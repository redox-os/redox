use alloc::boxed::Box;

use collections::string::{String, ToString};
use collections::vec::Vec;
use collections::vec_deque::VecDeque;

use core::ops::DerefMut;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use core::usize;

use common::elf::Elf;
use common::event::Event;
use common::get_slice::GetSlice;
use common::memory;

use scheduler::context::{context_switch, Context, ContextMemory};

use schemes::{Result, KScheme, Resource, ResourceSeek, Url};

use sync::Intex;

use syscall::SysError;
use syscall::handle::*;

#[derive(Copy, Clone, Debug)]
pub enum Msg {
    Start,
    Stop,
    Open(*const u8, usize),
    Close(usize),
    Dup(usize),
    Path(usize, *mut u8, usize),
    Read(usize, *mut u8, usize),
    Write(usize, *const u8, usize),
    Seek(usize, isize, isize),
    Sync(usize),
    Truncate(usize, usize),
    Event(*const Event),
}

pub struct Response {
    msg: Msg,
    result: AtomicUsize,
    ready: AtomicBool,
}

impl Response {
    pub fn new(msg: Msg) -> Box<Response> {
        box Response {
            msg: msg,
            result: AtomicUsize::new(0),
            ready: AtomicBool::new(false),
        }
    }

    pub fn set(&mut self, result: usize) {
        self.result.store(result, Ordering::SeqCst);
        self.ready.store(true, Ordering::SeqCst);
    }

    pub fn get(&self) -> usize {
        while !self.ready.load(Ordering::SeqCst) {
            unsafe { context_switch(false) };
        }

        return self.result.load(Ordering::SeqCst);
    }
}

impl Drop for Response {
    fn drop(&mut self) {
        while !self.ready.load(Ordering::SeqCst) {
            unsafe { context_switch(false) };
        }
    }
}

/// A scheme resource
pub struct SchemeResource {
    /// Pointer to parent
    pub parent: *mut SchemeItem,
    /// File handle
    pub handle: usize,
}

impl SchemeResource {
    pub fn send(&self, msg: Msg) -> Result<usize> {
        unsafe { (*self.parent).send(msg) }
    }
}

impl Resource for SchemeResource {
    /// Duplicate the resource
    fn dup(&self) -> Result<Box<Resource>> {
        match self.send(Msg::Dup(self.handle)) {
            Ok(fd) => Ok(box SchemeResource {
                parent: self.parent,
                handle: fd,
            }),
            Err(err) => Err(err)
        }
    }

    /// Return the url of this resource
    fn url(&self) -> Url {
        let mut buf: [u8; 4096] = [0; 4096];
        match self.send(Msg::Path(self.handle, buf.as_mut_ptr(), buf.len())) {
            Ok(result) => Url::from_string(unsafe {
                String::from_utf8_unchecked(Vec::from(buf.get_slice(None, Some(result))))
            }),
            Err(err) => Url::new()
        }
    }

    /// Read data to buffer
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut ptr = buf.as_mut_ptr();

        let contexts = ::env().contexts.lock();
        if let Some(current) = contexts.current() {
            if let Some(translated) = unsafe { current.translate(ptr as usize) } {
                ptr = translated as *mut u8;
            }
        }

        self.send(Msg::Read(self.handle, ptr, buf.len()))
    }

    /// Write to resource
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let mut ptr = buf.as_ptr();

        let contexts = ::env().contexts.lock();
        if let Some(current) = contexts.current() {
            if let Some(translated) = unsafe { current.translate(ptr as usize) } {
                ptr = translated as *const u8;
            }
        }

        self.send(Msg::Write(self.handle, ptr, buf.len()))
    }

    /// Seek
    fn seek(&mut self, pos: ResourceSeek) -> Result<usize> {
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

        self.send(Msg::Seek(self.handle, offset, whence))
    }

    /// Sync the resource
    fn sync(&mut self) -> Result<()> {
        match self.send(Msg::Sync(self.handle)) {
            Ok(_) => Ok(()),
            Err(err) => Err(err)
        }
    }

    fn truncate(&mut self, len: usize) -> Result<()> {
        match self.send(Msg::Truncate(self.handle, len)) {
            Ok(_) => Ok(()),
            Err(err) => Err(err)
        }
    }
}

impl Drop for SchemeResource {
    fn drop(&mut self) {
        self.send(Msg::Close(self.handle));
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
    /// Messages to be responded to
    responses: Intex<VecDeque<*mut Response>>,
    /// The handle
    handle: usize,
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
            responses: Intex::new(VecDeque::new()),
            handle: 0,
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
            if !part.is_empty() {
                scheme_item.scheme = part.to_string();
                break;
            }
        }

        let mut memory = Vec::new();
        if let Ok(mut resource) = scheme_item.binary.open() {
            let mut vec: Vec<u8> = Vec::new();
            resource.read_to_end(&mut vec);

            unsafe {
                let executable = Elf::from_data(vec.as_ptr() as usize);

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

                for segment in executable.load_segment().iter() {
                    let virtual_address = segment.vaddr as usize;
                    let virtual_size = segment.mem_len as usize;

                    //TODO: Warning: Investigate this hack!
                    let hack = virtual_address % 4096;

                    let physical_address = memory::alloc(virtual_size + hack);

                    if physical_address > 0 {
                        // Copy progbits
                        ::memcpy((physical_address + hack) as *mut u8,
                                 (executable.data + segment.off as usize) as *const u8,
                                 segment.file_len as usize);
                        // Zero bss
                        if segment.mem_len > segment.file_len {
                            ::memset((physical_address + hack + segment.file_len as usize) as *mut u8,
                                    0,
                                    segment.mem_len as usize - segment.file_len as usize);
                        }

                        memory.push(ContextMemory {
                            physical_address: physical_address,
                            virtual_address: virtual_address - hack,
                            virtual_size: virtual_size + hack,
                            writeable: segment.flags & 2 == 2
                        });
                    }
                }
            }
        }

        let wd = url.to_string();
        let scheme_item_ptr: *mut SchemeItem = scheme_item.deref_mut();
        Context::spawn(scheme_item.binary.to_string(),
                       box move || {
                           unsafe {
                               {
                                   let wd_c = wd + "\0";
                                   do_sys_chdir(wd_c.as_ptr());

                                   let stdio_c = "debug:\0";
                                   do_sys_open(stdio_c.as_ptr(), 0);
                                   do_sys_open(stdio_c.as_ptr(), 0);
                                   do_sys_open(stdio_c.as_ptr(), 0);

                                   let mut contexts = ::env().contexts.lock();
                                   if let Some(mut current) = contexts.current_mut() {
                                       current.unmap();
                                       (*current.memory.get()) = memory;
                                       current.map();
                                   }
                               }

                               (*scheme_item_ptr).run();
                           }
                       });

        scheme_item.handle = match scheme_item.send(Msg::Start) {
            Ok(handle) => handle,
            Err(_) => 0
        };

        scheme_item
    }
}

impl KScheme for SchemeItem {
    fn scheme(&self) -> &str {
        return &self.scheme;
    }

    // TODO: Hack for orbital
    fn event(&mut self, event: &Event) {
        self.send(Msg::Event(event));
    }

    fn open(&mut self, url: &Url, flags: usize) -> Result<Box<Resource>> {
        let c_str = url.to_string() + "\0";
        match self.send(Msg::Open(c_str.as_ptr(), flags)) {
            Ok(fd) => Ok(box SchemeResource {
                parent: self,
                handle: fd,
            }),
            Err(err) => Err(err)
        }
    }
}

impl Drop for SchemeItem {
    fn drop(&mut self) {
        self.send(Msg::Stop);
    }
}

impl SchemeItem {
    pub fn send(&mut self, msg: Msg) -> Result<usize> {
        let mut response = Response::new(msg);

        self.responses.lock().push_back(response.deref_mut());

        SysError::demux(response.get())
    }

    // TODO: More advanced check
    pub fn valid(&self, call: usize) -> bool {
        call > 0
    }

    pub fn run(&mut self) {
        let mut running = true;
        while running {
            let response_option = self.responses.lock().pop_front();

            if let Some(response_ptr) = response_option {
                let ret = match unsafe { (*response_ptr).msg } {
                    Msg::Start => if self.valid(self._start) {
                        let fn_ptr: *const usize = &self._start;
                        unsafe { (*(fn_ptr as *const extern "C" fn() -> usize))() }
                    } else {
                        0
                    },
                    Msg::Stop => if self.valid(self._stop) {
                        running = false;
                        let fn_ptr: *const usize = &self._stop;
                        unsafe { (*(fn_ptr as *const extern "C" fn(usize) -> usize))(self.handle) }
                    } else {
                        usize::MAX
                    },
                    Msg::Open(path, flags) => if self.valid(self._open) {
                        let fn_ptr: *const usize = &self._open;
                        unsafe { (*(fn_ptr as *const extern "C" fn(usize, *const u8, usize) -> usize))(self.handle, path, flags) }
                    } else {
                        usize::MAX
                    },
                    Msg::Event(event_ptr) => if self.valid(self._event) {
                        let fn_ptr: *const usize = &self._event;
                        unsafe { (*(fn_ptr as *const extern "C" fn(usize, usize) -> usize))(self.handle, event_ptr as usize) }
                    } else {
                        usize::MAX
                    },
                    Msg::Dup(fd) => if self.valid(self._dup) {
                        let fn_ptr: *const usize = &self._dup;
                        unsafe { (*(fn_ptr as *const extern "C" fn(usize) -> usize))(fd) }
                    } else {
                        usize::MAX
                    },
                    Msg::Path(fd, ptr, len) => if self.valid(self._fpath) {
                        let fn_ptr: *const usize = &self._fpath;
                        unsafe {
                            (*(fn_ptr as *const extern "C" fn(usize, *mut u8, usize) -> usize))(fd,
                                                                                                ptr,
                                                                                                len)
                        }
                    } else {
                        usize::MAX
                    },
                    Msg::Read(fd, ptr, len) => if self.valid(self._read) {
                        let fn_ptr: *const usize = &self._read;
                        unsafe {
                            (*(fn_ptr as *const extern "C" fn(usize, *mut u8, usize) -> usize))(fd,
                                                                                                ptr,
                                                                                                len)
                        }
                    } else {
                        usize::MAX
                    },
                    Msg::Write(fd, ptr, len) =>
                        if self.valid(self._write) {
                            let fn_ptr: *const usize = &self._write;
                            unsafe { (*(fn_ptr as *const extern "C" fn(usize, *const u8, usize) -> usize))(fd, ptr, len) }
                        } else {
                            usize::MAX
                        },
                    Msg::Seek(fd, offset, whence) =>
                        if self.valid(self._lseek) {
                            let fn_ptr: *const usize = &self._lseek;
                            unsafe { (*(fn_ptr as *const extern "C" fn(usize, isize, isize) -> usize))(fd, offset, whence) }
                        } else {
                            usize::MAX
                        },
                    Msg::Sync(fd) => if self.valid(self._fsync) {
                        let fn_ptr: *const usize = &self._fsync;
                        unsafe { (*(fn_ptr as *const extern "C" fn(usize) -> usize))(fd) }
                    } else {
                        usize::MAX
                    },
                    Msg::Truncate(fd, len) => if self.valid(self._ftruncate) {
                        let fn_ptr: *const usize = &self._ftruncate;
                        unsafe { (*(fn_ptr as *const extern "C" fn(usize, usize) -> usize))(fd, len) }
                    } else {
                        usize::MAX
                    },
                    Msg::Close(fd) => if self.valid(self._close) {
                        let fn_ptr: *const usize = &self._close;
                        unsafe { (*(fn_ptr as *const extern "C" fn(usize) -> usize))(fd) }
                    } else {
                        usize::MAX
                    },
                };

                unsafe { (*response_ptr).set(ret); }
            } else {
                unsafe { context_switch(false); }
            }
        }
    }
}
