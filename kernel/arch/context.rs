use common::slice::GetSlice;

use alloc::arc::Arc;
use alloc::boxed::{Box, FnBox};

use arch::memory;
use arch::paging::Page;
use arch::regs::Regs;

use collections::string::{String, ToString};
use collections::vec::Vec;

use common::time::Duration;

use core::cell::UnsafeCell;
use core::slice::{Iter, IterMut};
use core::{mem, ptr};
use core::ops::DerefMut;

use fs::Resource;

use syscall::{do_sys_exit, CLONE_FILES, CLONE_FS, CLONE_VM, CLONE_VFORK};

use system::error::{Error, Result, EBADF, EFAULT, ENOMEM, ESRCH, ENOENT, EINVAL};

use sync::WaitMap;

pub const CONTEXT_IMAGE_ADDR: usize = 0x8048000;
pub const CONTEXT_IMAGE_SIZE: usize = 0x10000000;

pub const CONTEXT_HEAP_ADDR: usize = CONTEXT_IMAGE_ADDR + CONTEXT_IMAGE_SIZE + memory::CLUSTER_SIZE;
pub const CONTEXT_HEAP_SIZE: usize = 0x40000000;

pub const CONTEXT_MMAP_ADDR: usize = CONTEXT_HEAP_ADDR + CONTEXT_HEAP_SIZE + memory::CLUSTER_SIZE;
pub const CONTEXT_MMAP_SIZE: usize = 0x20000000;

pub const CONTEXT_STACK_ADDR: usize = CONTEXT_MMAP_ADDR + CONTEXT_MMAP_SIZE + memory::CLUSTER_SIZE;
pub const CONTEXT_STACK_SIZE: usize = 0x100000;

pub struct ContextManager {
    pub inner: Vec<Box<Context>>,
    pub enabled: bool,
    pub i: usize,
    pub next_pid: usize,
}

impl ContextManager {
    pub fn new() -> ContextManager {
        ContextManager {
            inner: Vec::new(),
            enabled: false,
            i: 0,
            next_pid: 1,
        }
    }

    pub fn current(&self) -> Result<&Box<Context>> {
        let i = self.i;
        self.get(i)
    }

    pub fn current_mut(&mut self) -> Result<&mut Box<Context>> {
        let i = self.i;
        self.get_mut(i)
    }

    pub fn iter(&self) -> Iter<Box<Context>> {
        self.inner.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<Box<Context>> {
        self.inner.iter_mut()
    }

    pub fn get(&self, i: usize) -> Result<&Box<Context>> {
        if self.enabled {
            self.inner.get(i).ok_or(Error::new(ESRCH))
        } else{
            Err(Error::new(ESRCH))
        }
    }

    pub fn get_mut(&mut self, i: usize) -> Result<&mut Box<Context>> {
        if self.enabled {
            self.inner.get_mut(i).ok_or(Error::new(ESRCH))
        } else{
            Err(Error::new(ESRCH))
        }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub unsafe fn push(&mut self, context: Box<Context>) {
        self.inner.push(context);
    }

    pub unsafe fn clean(&mut self) {
        loop {
            if self.i >= self.len() {
                self.i -= self.len();
            }

            let mut remove = false;
            if let Ok(next) = self.current() {
                if next.exited {
                    remove = true;
                }
            }

            if remove {
                let i = self.i;
                drop(self.inner.remove(i));
            } else {
                break;
            }
        }

        if self.i >= self.len() {
            self.i -= self.len();
        }
    }
}

/// Switch context
///
/// Unsafe due to interrupt disabling, raw pointers, and unsafe Context functions
pub unsafe fn context_switch() {
    let mut current_ptr: *mut Context = 0 as *mut Context;
    let mut next_ptr: *mut Context = 0 as *mut Context;

    {
        let mut contexts = ::env().contexts.lock();
        if contexts.enabled {
            let current_i = contexts.i;
            'searching: loop {
                contexts.i += 1;
                contexts.clean();
                if let Ok(mut next) = contexts.current_mut() {
                    if next.blocked {
                        if let Some(wake) = next.wake {
                            if wake <= Duration::monotonic() {
                                next.blocked = false;
                                next.wake = None;
                                break 'searching;
                            }
                        }
                    } else {
                        break 'searching;
                    }
                }
                if contexts.i == current_i {
                    break 'searching;
                }
            }

            if contexts.i != current_i {
                if let Ok(mut current) = contexts.get_mut(current_i) {
                    current.unmap();

                    current_ptr = current.deref_mut();
                }

                if let Ok(mut next) = contexts.current_mut() {
                    next.switch += 1;

                    if let Some(ref mut tss) = ::TSS_PTR {
                        if next.kernel_stack > 0 {
                            tss.sp0 = next.kernel_stack + CONTEXT_STACK_SIZE - 128;
                        } else {
                            tss.sp0 = 0x200000 - 128;
                        }
                    }

                    next.map();

                    next_ptr = next.deref_mut();
                }
            }
        }
    }

    if current_ptr as usize > 0 && next_ptr as usize > 0 {
        (*current_ptr).switch_to(&mut *next_ptr);
    }
}

pub unsafe fn context_clone(regs: &Regs) -> Result<usize> {
    let mut contexts = ::env().contexts.lock();
    let flags = regs.bx;

    let kernel_stack = memory::alloc(CONTEXT_STACK_SIZE + 512);
    if kernel_stack > 0 {
        let clone_pid = Context::next_pid();

        let context = {
            let mut parent = try!(contexts.current_mut());

            //debugln!("{}: {}: clone to {}: {:X}", parent.pid, parent.name, clone_pid, flags);

            let regs_size = mem::size_of::<Regs>();
            let extra_size = mem::size_of::<usize>() * 3; /* Return pointer, interrupt code, regs pointer */
            let parent_regs_addr = (regs as *const Regs) as usize;
            let child_regs_addr = parent_regs_addr - parent.kernel_stack + kernel_stack;
            ::memcpy((child_regs_addr - extra_size) as *mut u8,
                     (parent_regs_addr - extra_size) as *const u8,
                     regs_size + extra_size);

            let child_regs = &mut *(child_regs_addr as *mut Regs);
            child_regs.ax = 0;

            let mut kernel_regs = parent.regs;
            kernel_regs.sp = child_regs_addr - extra_size;

            let fx = kernel_stack + CONTEXT_STACK_SIZE;
            ::memcpy(fx as *mut u8, parent.fx as *const u8, 512);

            box Context {
                pid: clone_pid,
                ppid: parent.pid,
                name: parent.name.clone(),
                iopl: parent.iopl,
                blocked: false,
                exited: false,
                switch: 0,
                time: 0,
                vfork: if flags & CLONE_VFORK == CLONE_VFORK {
                    parent.blocked = true;
                    Some(parent.deref_mut())
                } else {
                    None
                },
                wake: None,

                kernel_stack: kernel_stack,
                regs: kernel_regs,
                fx: fx,
                stack: if let Some(ref entry) = parent.stack {
                    let physical_address = memory::alloc(entry.virtual_size);
                    if physical_address > 0 {
                        ::memcpy(physical_address as *mut u8,
                                 entry.physical_address as *const u8,
                                 entry.virtual_size);
                        Some(ContextMemory {
                            physical_address: physical_address,
                            virtual_address: entry.virtual_address,
                            virtual_size: entry.virtual_size,
                            writeable: entry.writeable,
                            allocated: true,
                        })
                    } else {
                        None
                    }
                } else {
                    None
                },
                loadable: parent.loadable,

                image: if flags & CLONE_VM == CLONE_VM {
                    //debugln!("{}: {}: clone memory for {}", parent.pid, parent.name, clone_pid);

                    parent.image.clone()
                } else {
                    Arc::new(UnsafeCell::new((*parent.image.get()).dup()))
                },
                heap: if flags & CLONE_VM == CLONE_VM {
                    //debugln!("{}: {}: clone memory for {}", parent.pid, parent.name, clone_pid);

                    parent.heap.clone()
                } else {
                    Arc::new(UnsafeCell::new((*parent.heap.get()).dup()))
                },
                mmap: if flags & CLONE_VM == CLONE_VM {
                    //debugln!("{}: {}: clone memory for {}", parent.pid, parent.name, clone_pid);

                    parent.mmap.clone()
                } else {
                    Arc::new(UnsafeCell::new((*parent.mmap.get()).dup()))
                },
                env_vars: if flags & CLONE_VM == CLONE_VM {  // is CLONE_VM the good flag ?
                    parent.env_vars.clone()
                } else {
                    Arc::new(UnsafeCell::new((*parent.env_vars.get()).clone()))
                },
                cwd: if flags & CLONE_FS == CLONE_FS {
                    parent.cwd.clone()
                } else {
                    Arc::new(UnsafeCell::new((*parent.cwd.get()).clone()))
                },
                files: if flags & CLONE_FILES == CLONE_FILES {
                    //debugln!("{}: {}: clone resources for {}", parent.pid, parent.name, clone_pid);

                    parent.files.clone()
                } else {
                    let mut files: Vec<ContextFile> = Vec::new();
                    for file in (*parent.files.get()).iter() {
                        match file.resource.dup() {
                            Ok(resource) => {
                                //debugln!("{}: {}: dup resource {} for {}", parent.pid, parent.name, file.fd, clone_pid);

                                files.push(ContextFile {
                                    fd: file.fd,
                                    resource: resource,
                                });
                            },
                            Err(_err) => () //debugln!("{}: {}: failed to dup resource {} for {}: {}", parent.pid, parent.name, file.fd, clone_pid, err)
                        }
                    }
                    Arc::new(UnsafeCell::new(files))
                },

                statuses: WaitMap::new(),
            }
        };

        contexts.push(context);

        if flags & CLONE_VFORK == CLONE_VFORK {
            context_switch();
        }

        Ok(clone_pid)
    } else {
        Err(Error::new(ENOMEM))
    }
}

// Must have absolutely no pushes or pops
#[cfg(target_arch = "x86")]
#[allow(unused_variables)]
pub unsafe extern "cdecl" fn context_userspace(ip: usize,
                                               cs: usize,
                                               flags: usize,
                                               sp: usize,
                                               ss: usize) {
    asm!("mov eax, [esp + 16]
    mov ds, eax
    mov es, eax
    mov fs, eax
    mov gs, eax
    iretd" : : : "memory" : "intel", "volatile");
}

// Must have absolutely no pushes or pops
#[cfg(target_arch = "x86_64")]
#[allow(unused_variables)]
pub unsafe extern "cdecl" fn context_userspace(/*Throw away extra params from ABI*/ _rdi: usize, _rsi: usize, _rdx: usize, _rcx: usize, _r8: usize, _r9: usize,
                                               ip: usize,
                                               cs: usize,
                                               flags: usize,
                                               sp: usize,
                                               ss: usize) {
    asm!("mov rax, [esp + 32]
    mov ds, rax
    mov es, rax
    mov fs, rax
    mov gs, rax
    iretq" : : : "memory" : "intel", "volatile");
}

/// Reads a Boxed function and executes it
/// # Safety
/// Unsafe due to raw memory handling and FnBox
#[cfg(target_arch="x86")]
unsafe extern "cdecl" fn context_box(box_fn_ptr: usize) {
    let box_fn = ptr::read(box_fn_ptr as *mut Box<FnBox()>);
    memory::unalloc(box_fn_ptr);
    box_fn();
    do_sys_exit(0);
}

/// Reads a Boxed function and executes it
/// # Safety
/// Unsafe due to raw memory handling and FnBox
#[cfg(target_arch="x86_64")]
unsafe extern "cdecl" fn context_box(/*Throw away extra params from ABI*/ _rdi: usize, _rsi: usize, _rdx: usize, _rcx: usize, _r8: usize, _r9: usize,
                                    box_fn_ptr: usize) {
    let box_fn = ptr::read(box_fn_ptr as *mut Box<FnBox()>);
    memory::unalloc(box_fn_ptr);
    box_fn();
    do_sys_exit(0);
}

pub struct ContextMemory {
    pub physical_address: usize,
    pub virtual_address: usize,
    pub virtual_size: usize,
    pub writeable: bool,
    pub allocated: bool,
}

impl ContextMemory {
    pub unsafe fn map(&mut self) {
        for i in 0..(self.virtual_size + 4095) / 4096 {
            if self.writeable {
                Page::new(self.virtual_address + i * 4096)
                    .map_user_write(self.physical_address + i * 4096);
            } else {
                Page::new(self.virtual_address + i * 4096)
                    .map_user_read(self.physical_address + i * 4096);
            }
        }
    }

    pub unsafe fn unmap(&mut self) {
        for i in 0..(self.virtual_size + 4095) / 4096 {
            Page::new(self.virtual_address + i * 4096)
                .map_kernel_write(self.virtual_address + i * 4096);
        }
    }
}

impl Drop for ContextMemory {
    fn drop(&mut self) {
        if self.allocated {
            unsafe { memory::unalloc(self.physical_address) };
        }
    }
}

pub struct ContextFile {
    pub fd: usize,
    pub resource: Box<Resource>,
}

pub struct ContextZone {
    pub address: usize,
    pub size: usize,
    pub memory: Vec<ContextMemory>
}

impl ContextZone {
    pub fn new(address: usize, size: usize) -> ContextZone {
        ContextZone {
            address: address,
            size: size,
            memory: Vec::new()
        }
    }

    pub fn dup(&self) -> ContextZone {
        let mut mem: Vec<ContextMemory> = Vec::new();
        for entry in self.memory.iter() {
            let physical_address = unsafe { memory::alloc(entry.virtual_size) };
            if physical_address > 0 {
                //TODO: Remap pages during memcpy
                unsafe {
                    ::memcpy(physical_address as *mut u8,
                             entry.physical_address as *const u8,
                             entry.virtual_size);
                }

                //debugln!("{}: {}: dup memory {:X}:{:X} for {}", parent.pid, parent.name, entry.virtual_address, entry.virtual_address + entry.virtual_size, clone_pid);

                mem.push(ContextMemory {
                    physical_address: physical_address,
                    virtual_address: entry.virtual_address,
                    virtual_size: entry.virtual_size,
                    writeable: entry.writeable,
                    allocated: true,
                });
            } else {
                //debugln!("{}: {}: failed to dup memory {:X}:{:X} for {}", parent.pid, parent.name, entry.virtual_address, entry.virtual_address + entry.virtual_size, clone_pid);
            }
        }

        ContextZone {
            address: self.address,
            size: self.size,
            memory: mem
        }
    }

    pub fn size(&self) -> usize {
        let mut size = 0;

        for entry in self.memory.iter() {
            size += entry.virtual_size;
        }

        size
    }

    /// Get the next available memory map address
    pub fn next_mem(&self) -> usize {
        let mut next_mem = self.address;

        for mem in self.memory.iter() {
            let pages = (mem.virtual_size + 4095) / 4096;
            let end = mem.virtual_address + pages * 4096;
            if next_mem < end {
                next_mem = end;
            }
        }

        return next_mem;
    }

    /// Translate to physical if a ptr is inside of the mapped memory
    pub fn translate(&self, ptr: usize, len: usize) -> Option<usize> {
        for mem in self.memory.iter() {
            if ptr >= mem.virtual_address && ptr + len < mem.virtual_address + mem.virtual_size {
                return Some(ptr - mem.virtual_address + mem.physical_address);
            }
        }

        None
    }

    /// Get a memory map from a pointer
    pub fn get_mem<'a>(&'a self, ptr: usize) -> Result<&'a ContextMemory> {
        for mem in self.memory.iter() {
            if mem.virtual_address == ptr {
                return Ok(mem);
            }
        }

        Err(Error::new(ENOMEM))
    }

    /// Get a mutable memory map from a pointer
    pub fn get_mem_mut<'a>(&'a mut self, ptr: usize) -> Result<&'a mut ContextMemory> {
        for mem in self.memory.iter_mut() {
            if mem.virtual_address == ptr {
                return Ok(mem);
            }
        }

        Err(Error::new(ENOMEM))
    }

    /// Cleanup empty memory
    pub unsafe fn clean_mem(&mut self) {
        self.memory.retain(|mem| mem.virtual_size > 0);
    }

    pub unsafe fn map(&mut self) {
        for entry in self.memory.iter_mut() {
            entry.map();
        }
    }

    pub unsafe fn unmap(&mut self) {
        for entry in self.memory.iter_mut() {
            entry.unmap();
        }
    }
}

#[derive(Clone)]
pub struct EnvironmentVariable {
    name: String,
    value: String
}

pub struct Context {
    // These members are used for control purposes by the scheduler {
    // The PID of the context
    pub pid: usize,
    /// The PID of the parent
    pub ppid: usize,
    /// The name of the context
    pub name: String,
    /// The I/O privilege level
    pub iopl: usize,
    /// Indicates that the context is blocked, and should not be switched to
    pub blocked: bool,
    /// Indicates that the context exited
    pub exited: bool,
    /// How many times was the context switched to
    pub switch: usize,
    /// The number of time slices used
    pub time: usize,
    /// Indicates that the context needs to unblock parent
    pub vfork: Option<*mut Context>,
    /// When to wake up
    pub wake: Option<Duration>,
    // }

    // These members control the stack and registers and are unique to each context {
    // The kernel stack
    pub kernel_stack: usize,
    /// The current kernel registers
    pub regs: Regs,
    /// The location used to save and load SSE and FPU registers
    pub fx: usize,
    /// The context stack
    pub stack: Option<ContextMemory>,
    /// Indicates that registers can be loaded (they must be saved first)
    pub loadable: bool,
    // }

    // These members are cloned for threads, copied or created for processes {
    /// Program memory, cloned for threads, copied or created for processes. Modified by exec
    pub image: Arc<UnsafeCell<ContextZone>>,
    /// Heap, cloned for threads, copied or created for processes. Modified by memory allocation
    pub heap: Arc<UnsafeCell<ContextZone>>,
    /// Mmap memory, cloned for threads, copied or created for processes. Modified by mmap
    pub mmap: Arc<UnsafeCell<ContextZone>>,
    /// Environment variables, cloned for threads, copied or created for processes. Modified by set_env
    pub env_vars: Arc<UnsafeCell<Vec<EnvironmentVariable>>>,

    /// Program working directory, cloned for threads, copied or created for processes. Modified by chdir
    pub cwd: Arc<UnsafeCell<String>>,
    /// Program files, cloned for threads, copied or created for processes. Modified by file operations
    pub files: Arc<UnsafeCell<Vec<ContextFile>>>,
    // }

    /// Exit statuses of children
    pub statuses: WaitMap<usize, usize>,
}

impl Context {
    pub fn next_pid() -> usize {
        let mut contexts = ::env().contexts.lock();

        let mut next_pid = contexts.next_pid;

        let mut collision = true;
        while collision {
            collision = false;
            for context in contexts.iter() {
                if next_pid == context.pid {
                    next_pid += 1;
                    collision = true;
                    break;
                }
            }
        }

        let ret = next_pid;
        next_pid += 1;

        if next_pid >= 65536 {
            next_pid = 1;
        }

        contexts.next_pid = next_pid;

        ret
    }

    pub unsafe fn root() -> Box<Self> {
        let fx = memory::alloc(512);

        box Context {
            pid: Context::next_pid(),
            ppid: 0,
            name: "kidle".to_string(),
            iopl: 3,
            blocked: false,
            exited: false,
            switch: 0,
            time: 0,
            vfork: None,
            wake: None,

            kernel_stack: 0,
            regs: Regs::default(),
            fx: fx,
            stack: None,
            loadable: false,

            image: Arc::new(UnsafeCell::new(ContextZone::new(CONTEXT_IMAGE_ADDR, CONTEXT_IMAGE_SIZE))),
            heap: Arc::new(UnsafeCell::new(ContextZone::new(CONTEXT_HEAP_ADDR, CONTEXT_HEAP_SIZE))),
            mmap: Arc::new(UnsafeCell::new(ContextZone::new(CONTEXT_MMAP_ADDR, CONTEXT_MMAP_SIZE))),
            env_vars: Arc::new(UnsafeCell::new(Vec::new())),

            cwd: Arc::new(UnsafeCell::new(String::new())),
            files: Arc::new(UnsafeCell::new(Vec::new())),

            statuses: WaitMap::new(),
        }
    }

    pub unsafe fn new(name: String, call: usize, args: &Vec<usize>) -> Box<Self> {
        let kernel_stack = memory::alloc(CONTEXT_STACK_SIZE + 512);

        let mut regs = Regs::default();
        regs.sp = kernel_stack + CONTEXT_STACK_SIZE - 128;

        let fx = kernel_stack + CONTEXT_STACK_SIZE;

        let mut ret = box Context {
            pid: Context::next_pid(),
            ppid: 0,
            name: name,
            iopl: 3,
            blocked: false,
            exited: false,
            switch: 0,
            time: 0,
            vfork: None,
            wake: None,

            kernel_stack: kernel_stack,
            regs: regs,
            fx: fx,
            stack: None,
            loadable: false,

            image: Arc::new(UnsafeCell::new(ContextZone::new(CONTEXT_IMAGE_ADDR, CONTEXT_IMAGE_SIZE))),
            heap: Arc::new(UnsafeCell::new(ContextZone::new(CONTEXT_HEAP_ADDR, CONTEXT_HEAP_SIZE))),
            mmap: Arc::new(UnsafeCell::new(ContextZone::new(CONTEXT_MMAP_ADDR, CONTEXT_MMAP_SIZE))),
            env_vars: Arc::new(UnsafeCell::new(Vec::new())),

            cwd: Arc::new(UnsafeCell::new(String::new())),
            files: Arc::new(UnsafeCell::new(Vec::new())),

            statuses: WaitMap::new(),
        };

        for arg in args.iter() {
            ret.push(*arg);
        }

        ret.push(call);

        ret
    }

    pub fn spawn(name: String, box_fn: Box<FnBox()>) -> usize {
        let ret;

        unsafe {
            let box_fn_ptr: *mut Box<FnBox()> = memory::alloc_type();
            ptr::write(box_fn_ptr, box_fn);

            let mut context_box_args: Vec<usize> = Vec::new();
            context_box_args.push(box_fn_ptr as usize);
            context_box_args.push(0); //Return address, 0 catches bad code

            let context = Context::new(name, context_box as usize, &context_box_args);

            ret = context.pid;

            ::env().contexts.lock().push(context);
        }

        ret
    }

    pub fn canonicalize(&self, path: &str) -> String {
        if path.find(':').is_none() {
            let cwd = unsafe { &*self.cwd.get() };
            if path == "." {
                cwd.to_string()
            } else if path == ".." {
                cwd.get_slice(..cwd.get_slice(..cwd.len() - 1)
                                   .rfind('/')
                                   .map_or(cwd.len(), |i| i + 1))
                   .to_string()
            } else if path.starts_with("./") {
                cwd.to_string() + &path.get_slice(2..)
            } else if path.starts_with("../") {
                cwd.get_slice(..cwd.get_slice(..cwd.len() - 1)
                                   .rfind('/')
                                   .map_or(cwd.len(), |i| i + 1))
                   .to_string() + &path.get_slice(3..)
            } else if path.starts_with('/') {
                cwd.get_slice(..cwd.find(':').map_or(1, |i| i + 1)).to_string() + &path
            } else {
                cwd.to_string() + &path
            }
        } else {
            path.to_string()
        }
    }

    /// Get the next available file descriptor
    pub fn next_fd(&self) -> usize {
        let mut next_fd = 0;

        let mut collision = true;
        while collision {
            collision = false;
            for file in unsafe { (*self.files.get()).iter() } {
                if next_fd == file.fd {
                    next_fd = file.fd + 1;
                    collision = true;
                    break;
                }
            }
        }

        return next_fd;
    }

    /// Get a resource from a file descriptor
    pub fn get_file<'a>(&self, fd: usize) -> Result<&'a Box<Resource>> {
        for file in unsafe { (*self.files.get()).iter() } {
            if file.fd == fd {
                return Ok(&file.resource);
            }
        }

        //debugln!("{}: {}: file number {} not found", self.pid, self.name, fd);

        Err(Error::new(EBADF))
    }

    /// Get a mutable resource from a file descriptor
    pub fn get_file_mut<'a>(&mut self, fd: usize) -> Result<&'a mut Box<Resource>> {
        for file in unsafe { (*self.files.get()).iter_mut() } {
            if file.fd == fd {
                return Ok(&mut file.resource);
            }
        }

        //debugln!("{}: {}: file number {} not found", self.pid, self.name, fd);

        Err(Error::new(EBADF))
    }

    pub unsafe fn push(&mut self, data: usize) {
        self.regs.sp -= mem::size_of::<usize>();
        ptr::write(self.regs.sp as *mut usize, data);
    }

    /// Translate to physical if a ptr is inside of the mapped memory
    pub fn translate(&self, ptr: usize, len: usize) -> Result<usize> {
        if let Some(ref stack) = self.stack {
            if ptr >= stack.virtual_address && ptr + len <= stack.virtual_address + stack.virtual_size {
                return Ok(ptr - stack.virtual_address + stack.physical_address);
            }
        }

        if let Some(address) = unsafe { (*self.image.get()).translate(ptr, len) } {
            return Ok(address);
        }

        if let Some(address) = unsafe { (*self.heap.get()).translate(ptr, len) } {
            return Ok(address);
        }

        if let Some(address) = unsafe { (*self.mmap.get()).translate(ptr, len) } {
            return Ok(address);
        }

        Err(Error::new(EFAULT))
    }

    /// Gets an environment variable. Returns `None` if the variable is not defined
    pub fn get_env_var(&self, var_name: &str) -> Result<String> {
        for variable in unsafe { (*self.env_vars.get()).iter() } {
            if &variable.name == var_name {
                return Ok(variable.value.clone());
            }
        }
        Err(Error::new(ENOENT))
    }

    /// Sets an environment variable. Returns `None` if the variable name contains the `=`
    /// character
    pub fn set_env_var(&mut self, name: &str, value: &str) -> Result<()> {
        if name.contains('=') {
            return Err(Error::new(EINVAL));
        }

        for mut variable in unsafe { (*self.env_vars.get()).iter_mut() } {
            if &variable.name == name {
                variable.value = String::from(value);
                return Ok(());
            }
        }
        unsafe { (*self.env_vars.get()).push(EnvironmentVariable { name: String::from(name), value: String::from(value) }) };
        Ok(())
    }

    pub fn list_env_vars(&self) -> Result<Vec<(String, String)>> {
        let mut vars_buf = Vec::new();
        for ref variable in unsafe { (*self.env_vars.get()).iter() } {
            vars_buf.push((variable.name.clone(), variable.value.clone()));
        }
        Ok(vars_buf)
    }

    pub unsafe fn map(&mut self) {
        if let Some(ref mut stack) = self.stack {
            stack.map();
        }
        (*self.image.get()).map();
        (*self.heap.get()).map();
        (*self.mmap.get()).map();
    }

    pub unsafe fn unmap(&mut self) {
        (*self.mmap.get()).unmap();
        (*self.heap.get()).unmap();
        (*self.image.get()).unmap();
        if let Some(ref mut stack) = self.stack {
            stack.unmap();
        }
    }

    // This function must not push or pop
    #[cfg(target_arch = "x86")]
    #[cold]
    #[inline(never)]
    #[naked]
    pub unsafe fn switch_to(&mut self, next: &mut Context) {
        //asm!("xchg bx, bx" : : : "memory" : "intel", "volatile");

        asm!("fxsave [$0]" : : "r"(self.fx) : "memory" : "intel", "volatile");
        self.loadable = true;
        if next.loadable {
            asm!("fxrstor [$0]" : : "r"(next.fx) : "memory" : "intel", "volatile");
        }else{
            asm!("fninit" : : : "memory" : "intel", "volatile");
        }

        asm!("pushfd ; pop $0" : "=r"(self.regs.flags) : : "memory" : "intel", "volatile");
        asm!("push $0 ; popfd" : : "r"(next.regs.flags) : "memory" : "intel", "volatile");

        asm!("mov $0, ebx" : "=r"(self.regs.bx) : : "memory" : "intel", "volatile");
        asm!("mov ebx, $0" : : "r"(next.regs.bx) : "memory" : "intel", "volatile");

        asm!("mov $0, edi" : "=r"(self.regs.di) : : "memory" : "intel", "volatile");
        asm!("mov edi, $0" : : "r"(next.regs.di) : "memory" : "intel", "volatile");

        asm!("mov $0, esi" : "=r"(self.regs.si) : : "memory" : "intel", "volatile");
        asm!("mov esi, $0" : : "r"(next.regs.si) : "memory" : "intel", "volatile");

        asm!("mov $0, ebp" : "=r"(self.regs.bp) : : "memory" : "intel", "volatile");
        asm!("mov ebp, $0" : : "r"(next.regs.bp) : "memory" : "intel", "volatile");

        asm!("mov $0, esp" : "=r"(self.regs.sp) : : "memory" : "intel", "volatile");
        asm!("mov esp, $0" : : "r"(next.regs.sp) : "memory" : "intel", "volatile");
    }

    // This function must not push or pop
    #[cfg(target_arch = "x86_64")]
    #[cold]
    #[inline(never)]
    #[naked]
    pub unsafe fn switch_to(&mut self, next: &mut Context) {
        //asm!("xchg bx, bx" : : : "memory" : "intel", "volatile");

        asm!("fxsave [$0]" : : "r"(self.fx) : "memory" : "intel", "volatile");
        self.loadable = true;
        if next.loadable {
            asm!("fxrstor [$0]" : : "r"(next.fx) : "memory" : "intel", "volatile");
        }else{
            asm!("fninit" : : : "memory" : "intel", "volatile");
        }

        asm!("pushfq ; pop $0" : "=r"(self.regs.flags) : : "memory" : "intel", "volatile");
        asm!("push $0 ; popfq" : : "r"(next.regs.flags) : "memory" : "intel", "volatile");

        asm!("mov $0, rbx" : "=r"(self.regs.bx) : : "memory" : "intel", "volatile");
        asm!("mov rbx, $0" : : "r"(next.regs.bx) : "memory" : "intel", "volatile");

        asm!("mov $0, r12" : "=r"(self.regs.r12) : : "memory" : "intel", "volatile");
        asm!("mov r12, $0" : : "r"(next.regs.r12) : "memory" : "intel", "volatile");

        asm!("mov $0, r13" : "=r"(self.regs.r13) : : "memory" : "intel", "volatile");
        asm!("mov r13, $0" : : "r"(next.regs.r13) : "memory" : "intel", "volatile");

        asm!("mov $0, r14" : "=r"(self.regs.r14) : : "memory" : "intel", "volatile");
        asm!("mov r14, $0" : : "r"(next.regs.r14) : "memory" : "intel", "volatile");

        asm!("mov $0, r15" : "=r"(self.regs.r15) : : "memory" : "intel", "volatile");
        asm!("mov r15, $0" : : "r"(next.regs.r15) : "memory" : "intel", "volatile");

        asm!("mov $0, rbp" : "=r"(self.regs.bp) : : "memory" : "intel", "volatile");
        asm!("mov rbp, $0" : : "r"(next.regs.bp) : "memory" : "intel", "volatile");

        asm!("mov $0, rsp" : "=r"(self.regs.sp) : : "memory" : "intel", "volatile");
        asm!("mov rsp, $0" : : "r"(next.regs.sp) : "memory" : "intel", "volatile");
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        if let Some(vfork) = self.vfork.take() {
            unsafe { (*vfork).blocked = false; }
        }
        if self.kernel_stack > 0 {
            unsafe { memory::unalloc(self.kernel_stack); }
        }
    }
}
