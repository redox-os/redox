use common::slice::GetSlice;

use alloc::arc::Arc;
use alloc::boxed::{Box, FnBox};

use arch::memory;
use arch::paging::Page;

use collections::string::{String, ToString};
use collections::vec::Vec;

use core::cell::UnsafeCell;
use core::slice::{Iter, IterMut};
use core::{mem, ptr};
use core::ops::DerefMut;

use schemes::Resource;

use syscall::{CLONE_FILES, CLONE_FS, CLONE_VM};
use syscall::handle::do_sys_exit;

use memcpy;
use env;

pub const CONTEXT_STACK_SIZE: usize = 1024 * 1024;
pub const CONTEXT_STACK_ADDR: usize = 0x70000000;
pub const CONTEXT_SLICES: usize = 4;

pub struct ContextManager<'a> {
    pub inner: Vec<Context<'a>>,
    pub enabled: bool,
    pub i: usize,
    pub next_pid: usize,
}

impl<'a> ContextManager<'a> {
    pub fn new() -> ContextManager<'a> {
        ContextManager {
            inner: Vec::new(),
            enabled: false,
            i: 0,
            next_pid: 1,
        }
    }

    pub fn current(&self) -> Option<&Context> {
        let i = self.i;
        self.get(i)
    }

    pub fn current_mut(&mut self) -> Option<&mut Context<'a>> {
        let i = self.i;
        self.get_mut(i)
    }

    pub fn iter(&self) -> Iter<Context<'a>> {
        self.inner.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<Context<'a>> {
        self.inner.iter_mut()
    }

    pub fn get(&self, i: usize) -> Option<&Context> {
        if self.enabled {
            self.inner.get(i)
        } else{
            None
        }
    }

    pub fn get_mut(&mut self, i: usize) -> Option<&mut Context<'a>> {
        if self.enabled {
            self.inner.get_mut(i)
        } else{
            None
        }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub unsafe fn push(&mut self, context: Context<'a>) {
        self.inner.push(context);
    }

    pub unsafe fn clean(&mut self) {
        loop {
            if self.i >= self.len() {
                self.i -= self.len();
            }

            let mut remove = false;
            if let Some(next) = self.current() {
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
pub unsafe fn context_switch(interrupted: bool) {
    let mut current_ptr: *mut Context = 0 as *mut Context;
    let mut next_ptr: *mut Context = 0 as *mut Context;

    {
        let mut contexts = ::env().contexts.lock();
        if contexts.enabled {
            let current_i = contexts.i;
            contexts.i += 1;
            contexts.clean();

            if contexts.i != current_i {
                if let Some(mut current) = contexts.get_mut(current_i) {
                    current.interrupted = interrupted;

                    current.unmap();

                    current_ptr = current.deref_mut();
                }

                if let Some(mut next) = contexts.current_mut() {
                    next.interrupted = false;
                    next.slices = CONTEXT_SLICES;

                    if next.kernel_stack > 0 {
                        if let Some(ref mut tss) = ::TSS_PTR {
                            tss.sp0 = next.kernel_stack + CONTEXT_STACK_SIZE - 128;
                        }
                    } else {
                        if let Some(ref mut tss) = ::TSS_PTR {
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

/// Clone context
///
/// Unsafe due to interrupt disabling, C memory handling, and raw pointers
pub unsafe extern "cdecl" fn context_clone(parent_ptr: *const Context,
                                           flags: usize,
                                           clone_pid: usize) {
    {
        let mut contexts = env().contexts.lock();

        let kernel_stack = memory::alloc(CONTEXT_STACK_SIZE + 512);
        if kernel_stack > 0 {

            memcpy(kernel_stack as *mut u8,
                   (*parent_ptr).kernel_stack as *const u8,
                   CONTEXT_STACK_SIZE + 512);

            let context = Context {
                pid: clone_pid,
                ppid: (*parent_ptr).pid,
                name: &(*parent_ptr).name.to_string(),
                interrupted: (*parent_ptr).interrupted,
                exited: (*parent_ptr).exited,
                slices: CONTEXT_SLICES,
                slice_total: 0,

                kernel_stack: kernel_stack,
                sp: (*parent_ptr).sp - (*parent_ptr).kernel_stack + kernel_stack,
                flags: (*parent_ptr).flags,
                fx: kernel_stack + CONTEXT_STACK_SIZE,
                stack: if let Some(ref entry) = (*parent_ptr).stack {
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
                loadable: (*parent_ptr).loadable,

                cwd: if flags & CLONE_FS == CLONE_FS {
                    (*parent_ptr).cwd
                } else {
                    Arc::new(UnsafeCell::new(*((*parent_ptr).cwd.get())))
                },
                memory: if flags & CLONE_VM == CLONE_VM {
                    (*parent_ptr).memory.clone()
                } else {
                    let mut mem: Vec<ContextMemory> = Vec::new();
                    for entry in (*(*parent_ptr).memory.get()).iter() {
                        let physical_address = memory::alloc(entry.virtual_size);
                        if physical_address > 0 {
                            ::memcpy(physical_address as *mut u8,
                                     entry.physical_address as *const u8,
                                     entry.virtual_size);
                            mem.push(ContextMemory {
                                physical_address: physical_address,
                                virtual_address: entry.virtual_address,
                                virtual_size: entry.virtual_size,
                                writeable: entry.writeable,
                                allocated: true,
                            });
                        }
                    }
                    Arc::new(UnsafeCell::new(mem))
                },
                files: if flags & CLONE_FILES == CLONE_FILES {
                    (*parent_ptr).files.clone()
                } else {
                    let mut files: Vec<ContextFile> = Vec::new();
                    for file in (*(*parent_ptr).files.get()).iter() {
                        if let Ok(resource) = file.resource.dup() {
                            files.push(ContextFile {
                                fd: file.fd,
                                resource: resource,
                            });
                        }
                    }
                    Arc::new(UnsafeCell::new(files))
                },

                statuses: Vec::new(),
            };

            contexts.push(context);
        }
    }


    do_sys_exit(0);
}

// Must have absolutely no pushes or pops
#[cfg(target_arch = "x86")]
#[allow(unused_variables)]
pub unsafe extern "cdecl" fn context_userspace(ip: usize,
                                               cs: usize,
                                               flags: usize,
                                               sp: usize,
                                               ss: usize) {
    asm!("xchg bx, bx
    mov eax, [esp + 16]
    mov ds, eax
    mov es, eax
    mov fs, eax
    mov gs, eax
    iretd" : : : "memory" : "intel", "volatile");
}

// Must have absolutely no pushes or pops
#[cfg(target_arch = "x86_64")]
#[allow(unused_variables)]
pub unsafe extern "cdecl" fn context_userspace(ip: usize,
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
///
/// Unsafe due to raw memory handling and FnBox
pub unsafe extern "cdecl" fn context_box(box_fn_ptr: usize) {
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
            Page::new(self.virtual_address + i * 4096).map_identity();
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

pub struct ContextStatus {
    pub pid: usize,
    pub status: usize,
}

pub struct Context<'a> {
// These members are used for control purposes by the scheduler {
// The PID of the context
    pub pid: usize,
    /// The PID of the parent
    pub ppid: usize,
    /// The name of the context
    pub name: &'a str,
    /// Indicates that the context was interrupted, used for prioritizing active contexts
    pub interrupted: bool,
    /// Indicates that the context exited
    pub exited: bool,
    /// The number of time slices left
    pub slices: usize,
    /// The total of all used slices
    pub slice_total: usize,
// }

// These members control the stack and registers and are unique to each context {
// The kernel stack
    pub kernel_stack: usize,
    /// The current kernel stack pointer
    pub sp: usize,
    /// The current kernel flags
    pub flags: usize,
    /// The location used to save and load SSE and FPU registers
    pub fx: usize,
    /// The context stack
    pub stack: Option<ContextMemory>,
    /// Indicates that registers can be loaded (they must be saved first)
    pub loadable: bool,
// }

// These members are cloned for threads, copied or created for processes {
/// Program working directory, cloned for threads, copied or created for processes. Modified by chdir
    pub cwd: Arc<UnsafeCell<String>>,
/// Program memory, cloned for threads, copied or created for processes. Modified by memory allocation
    pub memory: Arc<UnsafeCell<Vec<ContextMemory>>>,
/// Program files, cloned for threads, copied or created for processes. Modified by file operations
    pub files: Arc<UnsafeCell<Vec<ContextFile>>>,
// }

/// Exit statuses of children
    pub statuses: Vec<ContextStatus>,
}

impl<'a> Context<'a> {
    pub unsafe fn next_pid() -> usize {
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

    pub unsafe fn root() -> Self {
        Context {
            pid: Context::next_pid(),
            ppid: 0,
            name: "kidle",
            interrupted: false,
            exited: false,
            slices: CONTEXT_SLICES,
            slice_total: 0,

            kernel_stack: 0,
            sp: 0,
            flags: 0,
            fx: memory::alloc(512),
            stack: None,
            loadable: false,

            cwd: Arc::new(UnsafeCell::new(String::new())),
            memory: Arc::new(UnsafeCell::new(Vec::new())),
            files: Arc::new(UnsafeCell::new(Vec::new())),

            statuses: Vec::new(),
        }
    }

    pub unsafe fn new(name: &'a str, call: usize, args: &Vec<usize>) -> Self {
        let kernel_stack = memory::alloc(CONTEXT_STACK_SIZE + 512);

        let mut ret = Context {
            pid: Context::next_pid(),
            ppid: 0,
            name: name,
            interrupted: false,
            exited: false,
            slices: CONTEXT_SLICES,
            slice_total: 0,

            kernel_stack: kernel_stack,
            sp: kernel_stack + CONTEXT_STACK_SIZE - 128,
            flags: 0,
            fx: kernel_stack + CONTEXT_STACK_SIZE,
            stack: None,
            loadable: false,

            cwd: Arc::new(UnsafeCell::new(String::new())),
            memory: Arc::new(UnsafeCell::new(Vec::new())),
            files: Arc::new(UnsafeCell::new(Vec::new())),

            statuses: Vec::new(),
        };

        for arg in args.iter() {
            ret.push(*arg);
        }

        ret.push(call);

        ret
    }

    pub fn spawn<F: FnBox()>(name: &'a str, fn_: F) -> usize {
        let ret;

        unsafe {
            let fn_ptr: *mut Box<FnBox()> = memory::alloc_type();
            ptr::write(fn_ptr, box fn_);

            let mut context_args = Vec::new();
            context_args.push(fn_ptr as usize);
            context_args.push(0); // Return address, 0 catches bad code

            let context = Context::new(&name.to_string(), fn_ptr as usize, &context_args);

            ret = context.pid;

            ::env().contexts.lock().push(context);
        }

        ret
    }

    pub unsafe fn canonicalize(&self, path: &str) -> String {
        if path.find(':').is_none() {
            let cwd = &*self.cwd.get();
            if path.starts_with("../") {
                cwd.get_slice(..cwd.get_slice(..cwd.len() - 1)
                                   .rfind('/')
                                   .map_or(cwd.len(), |i| i + 1))
                   .to_string() + &path.get_slice(3..)
            } else if path.starts_with("./") {
                cwd.to_string() + &path.get_slice(2..)
            } else if path.starts_with('/') {
                cwd.get_slice(..cwd.find(':').map_or(1, |i| i + 1)).to_string() + &path
            } else {
                cwd.to_string() + &path
            }
        } else {
            path.to_string()
        }
    }

    /// Get the next available memory map address
    pub unsafe fn next_mem(&self) -> usize {
        let mut next_mem = 0;

        for mem in (*self.memory.get()).iter() {
            let pages = (mem.virtual_size + 4095) / 4096;
            let end = mem.virtual_address + pages * 4096;
            if next_mem < end {
                next_mem = end;
            }
        }

        return next_mem;
    }

    /// Translate to physical if a ptr is inside of the mapped memory
    pub unsafe fn translate(&self, ptr: usize) -> Option<usize> {
        if let Some(ref stack) = self.stack {
            if ptr >= stack.virtual_address && ptr < stack.virtual_address + stack.virtual_size {
                return Some(ptr - stack.virtual_address + stack.physical_address);
            }
        }

        for mem in (*self.memory.get()).iter() {
            if ptr >= mem.virtual_address && ptr < mem.virtual_address + mem.virtual_size {
                return Some(ptr - mem.virtual_address + mem.physical_address);
            }
        }

        None
    }

    /// Get a memory map from a pointer
    pub unsafe fn get_mem(&self, ptr: usize) -> Option<&ContextMemory> {
        for mem in (*self.memory.get()).iter() {
            if mem.virtual_address == ptr {
                return Some(mem);
            }
        }

        None
    }

    /// Get a mutable memory map from a pointer
    pub unsafe fn get_mem_mut(&mut self, ptr: usize) -> Option<&mut ContextMemory> {
        for mem in (*self.memory.get()).iter_mut() {
            if mem.virtual_address == ptr {
                return Some(mem);
            }
        }

        None
    }

    /// Cleanup empty memory
    pub unsafe fn clean_mem(&mut self) {
        let mut i = 0;
        while i < (*self.memory.get()).len() {
            let mut remove = false;
            if let Some(mem) = (*self.memory.get()).get(i) {
                if mem.virtual_size == 0 {
                    remove = true;
                }
            }

            if remove {
                drop((*self.memory.get()).remove(i));
            } else {
                i += 1;
            }
        }
    }

    /// Get the next available file descriptor
    pub unsafe fn next_fd(&self) -> usize {
        let mut next_fd = 0;

        let mut collision = true;
        while collision {
            collision = false;
            for file in (*self.files.get()).iter() {
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
    pub unsafe fn get_file(&self, fd: usize) -> Option<&Box<Resource>> {
        for file in (*self.files.get()).iter() {
            if file.fd == fd {
                return Some(&file.resource);
            }
        }

        None
    }

    /// Get a mutable resource from a file descriptor
    pub unsafe fn get_file_mut(&mut self, fd: usize) -> Option<&mut Box<Resource>> {
        for file in (*self.files.get()).iter_mut() {
            if file.fd == fd {
                return Some(&mut file.resource);
            }
        }

        None
    }

    pub unsafe fn push(&mut self, data: usize) {
        self.sp -= mem::size_of::<usize>();
        ptr::write(self.sp as *mut usize, data);
    }

    pub unsafe fn map(&mut self) {
        if let Some(ref mut stack) = self.stack {
            stack.map();
        }
        for entry in (*self.memory.get()).iter_mut() {
            entry.map();
        }
    }

    pub unsafe fn unmap(&mut self) {
        for entry in (*self.memory.get()).iter_mut() {
            entry.unmap();
        }
        if let Some(ref mut stack) = self.stack {
            stack.unmap();
        }
    }

    // This function must not push or pop
    #[cfg(target_arch = "x86")]
    #[cold]
    #[inline(never)]
    pub unsafe fn switch_to(&mut self, next: &mut Context) {
        asm!("pushfd
            pop $0"
            : "=r"(self.flags)
            :
            : "memory"
            : "intel", "volatile");

        asm!(""
            : "={esp}"(self.sp)
            :
            : "memory"
            : "intel", "volatile");

        asm!("fxsave [$0]"
            :
            : "r"(self.fx)
            : "memory"
            : "intel", "volatile");

        self.loadable = true;

        if next.loadable {
            asm!("fxrstor [$0]"
                :
                : "r"(next.fx)
                : "memory"
                : "intel", "volatile");
        }

        asm!(""
            :
            : "{esp}"(next.sp)
            : "memory"
            : "intel", "volatile");


        asm!("push $0
            popfd"
            :
            : "r"(next.flags)
            : "memory"
            : "intel", "volatile");
    }

    // This function must not push or pop
    #[cfg(target_arch = "x86_64")]
    #[cold]
    #[inline(never)]
    pub unsafe fn switch_to(&mut self, next: &mut Context) {
        asm!("pushfq
            pop $0"
            : "=r"(self.flags)
            :
            : "memory"
            : "intel", "volatile");

        asm!(""
            : "={rsp}"(self.sp)
            :
            : "memory"
            : "intel", "volatile");

        asm!("fxsave [$0]"
            :
            : "r"(self.fx)
            : "memory"
            : "intel", "volatile");

        self.loadable = true;

        if next.loadable {
            asm!("fxrstor [$0]"
                :
                : "r"(next.fx)
                : "memory"
                : "intel", "volatile");
        }

        asm!(""
            :
            : "{rsp}"(next.sp)
            : "memory"
            : "intel", "volatile");


        asm!("push $0
            popfq"
            :
            : "r"(next.flags)
            : "memory"
            : "intel", "volatile");
    }
}

impl<'a> Drop for Context<'a> {
    fn drop(&mut self) {
        if self.kernel_stack > 0 {
            unsafe { memory::unalloc(self.kernel_stack) };
        }
    }
}
