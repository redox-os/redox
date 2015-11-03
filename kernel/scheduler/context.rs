use alloc::boxed::{Box, FnBox};
use alloc::rc::Rc;

use collections::string::{String, ToString};
use collections::vec::Vec;

use core::cell::UnsafeCell;
use core::{mem, ptr};

use common::debug;
use common::memory;
use common::paging::Page;
use scheduler;

use schemes::Resource;

use syscall::common::{CLONE_FILES, CLONE_FS, CLONE_VM, Regs, SYS_YIELD};

pub const CONTEXT_STACK_SIZE: usize = 1024 * 1024;

pub static mut contexts_ptr: *mut Vec<Box<Context>> = 0 as *mut Vec<Box<Context>>;
pub static mut context_i: usize = 0;
pub static mut context_enabled: bool = false;

/// This provides a way to yield inside of the kernel
///
/// It should only be used when absolutely necessary
pub unsafe fn recursive_unsafe_yield(){
    asm!("int 0x80" : : "{eax}"(SYS_YIELD) : : "intel", "volatile");
}

/// Switch context
///
/// Unsafe due to interrupt disabling, raw pointers, and unsafe Context functions
pub unsafe fn context_switch(regs: &mut Regs, interrupted: bool) {
    let reenable = scheduler::start_no_ints();

    let contexts = &mut *contexts_ptr;
    if context_enabled {
        let current_i = context_i;
        context_i += 1;
        // The only garbage collection in Redox
        loop {
            if context_i >= contexts.len() {
                context_i -= contexts.len();
            }

            let mut remove = false;
            if let Some(next) = contexts.get(context_i) {
                if next.exited {
                    remove = true;
                }
            }

            if remove {
                drop(contexts.remove(context_i));
            } else {
                break;
            }
        }

        if context_i >= contexts.len() {
            context_i -= contexts.len();
        }

        if context_i != current_i {
            if let Some(current) = contexts.get(current_i) {
                if let Some(next) = contexts.get(context_i) {
                    let current_ptr: *mut Box<Context> = mem::transmute(current as *const Box<Context>);
                    let next_ptr: *mut Box<Context> = mem::transmute(next as *const Box<Context>);

                    (*current_ptr).interrupted = interrupted;
                    (*next_ptr).interrupted = false;

                    (*current_ptr).unmap();
                    (*next_ptr).map();

                    (*current_ptr).switch_fx(&mut *next_ptr);
                    (*current_ptr).switch_stack(&mut *next_ptr);
                }
            }
        }
    }

    scheduler::end_no_ints(reenable);
}

/// Clone context
///
/// Unsafe due to interrupt disabling, C memory handling, and raw pointers
pub unsafe extern "cdecl" fn context_clone(parent_ptr: *const Context, flags: usize){
    let reenable = scheduler::start_no_ints();

    debug::debug(&format!("Parent During: {:X} {} {:X}\n", parent_ptr as usize, Context::current_i(), flags));

    let stack = memory::alloc(CONTEXT_STACK_SIZE + 512);
    if stack > 0 {
        let parent = &*parent_ptr;

        ::memcpy(stack as *mut u8, parent.stack as *const u8, CONTEXT_STACK_SIZE + 512);

        let mut context = box Context {
            interrupted: parent.interrupted,
            exited: parent.exited,

            regs: parent.regs,
            stack: stack,
            fx: stack + CONTEXT_STACK_SIZE,
            fx_enabled: parent.fx_enabled,

            args: parent.args.clone(),
            cwd: if flags & CLONE_FS == CLONE_FS {
                parent.cwd.clone()
            } else {
                Rc::new(UnsafeCell::new((*parent.cwd.get()).clone()))
            },
            memory: if flags & CLONE_VM == CLONE_VM {
                parent.memory.clone()
            } else {
                let mut mem: Vec<ContextMemory> = Vec::new();
                for entry in (*parent.memory.get()).iter() {
                    let physical_address = memory::alloc(entry.virtual_size);
                    if physical_address > 0 {
                        ::memcpy(physical_address as *mut u8, entry.physical_address as *const u8, entry.virtual_size);
                        mem.push(ContextMemory {
                            physical_address: physical_address,
                            virtual_address: entry.virtual_address,
                            virtual_size: entry.virtual_size,
                        });
                    }
                }
                Rc::new(UnsafeCell::new(mem))
            },
            files: if flags & CLONE_FILES == CLONE_FILES {
                parent.files.clone()
            }else {
                let mut files: Vec<ContextFile> = Vec::new();
                for file in (*parent.files.get()).iter() {
                    if let Some(resource) = file.resource.dup() {
                        files.push(ContextFile {
                            fd: file.fd,
                            resource: resource
                        });
                    }
                }
                Rc::new(UnsafeCell::new(files))
            },
        };

        context.regs.sp = stack + (parent.regs.sp - parent.stack);

        let contexts = &mut *contexts_ptr;
        contexts.push(context);
    }

    scheduler::end_no_ints(reenable);
}

//TODO: To clean up memory leak, current must be destroyed!
/// Exit context
///
/// Unsafe due to interrupt disabling and raw pointers
pub unsafe fn context_exit() {
    let reenable = scheduler::start_no_ints();

    if let Some(mut current) = Context::current_mut() {
        current.exited = true;
    }

    scheduler::end_no_ints(reenable);

    recursive_unsafe_yield();
}

// Currently unused?
/// Reads a Boxed function and executes it
///
/// Unsafe due to raw memory handling and FnBox
pub unsafe extern "cdecl" fn context_box(box_fn_ptr: usize) {
    let box_fn = ptr::read(box_fn_ptr as *mut Box<FnBox()>);
    memory::unalloc(box_fn_ptr);
    box_fn();
}

pub struct ContextMemory {
    pub physical_address: usize,
    pub virtual_address: usize,
    pub virtual_size: usize,
}

impl ContextMemory {
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

impl Drop for ContextMemory {
    fn drop(&mut self) {
        unsafe { memory::unalloc(self.physical_address) };
    }
}

pub struct ContextFile {
    pub fd: usize,
    pub resource: Box<Resource>,
}

pub struct Context {
    /* These members are used for control purposes by the scheduler { */
        /// Indicates that the context was interrupted, used for prioritizing active contexts
        pub interrupted: bool,
        /// Indicates that the context exited and needs to be cleaned up
        pub exited: bool,
    /* } */

    /* These members control the stack and registers and are unique to each context { */
        /// The context registers
        pub regs: Regs,
        /// The context stack
        pub stack: usize,
        /// The location used to save and load SSE and FPU registers
        pub fx: usize,
        /// Indicates that fx can be loaded (it must be saved first)
        pub fx_enabled: bool,
    /* } */

    /* These members are cloned for threads, copied or created for processes { */
        /// Program arguments, cloned for threads, copied or created for processes. It is usually read-only, but is modified by execute
        pub args: Rc<UnsafeCell<Vec<String>>>,
        /// Program working directory, cloned for threads, copied or created for processes. Modified by chdir
        pub cwd: Rc<UnsafeCell<String>>,
        /// Program memory, cloned for threads, copied or created for processes. Modified by memory allocation
        pub memory: Rc<UnsafeCell<Vec<ContextMemory>>>,
        /// Program files, cloned for threads, copied or created for processes. Modified by file operations
        pub files: Rc<UnsafeCell<Vec<ContextFile>>>,
    /* } */
}

impl Context {
    pub unsafe fn root() -> Box<Self> {
        box Context {
            interrupted: false,
            exited: false,

            regs: Regs::default(),
            stack: 0,
            fx: memory::alloc(512),
            fx_enabled: false,

            args: Rc::new(UnsafeCell::new(Vec::new())),
            cwd: Rc::new(UnsafeCell::new(String::new())),
            memory: Rc::new(UnsafeCell::new(Vec::new())),
            files: Rc::new(UnsafeCell::new(Vec::new())),
        }
    }

    #[cfg(target_arch = "x86")]
    pub unsafe fn new(call: usize, args: &Vec<usize>) -> Box<Self> {
        let stack = memory::alloc(CONTEXT_STACK_SIZE + 512);

        let mut ret = box Context {
            interrupted: false,
            exited: false,

            regs: Regs::default(),
            stack: stack,
            fx: stack + CONTEXT_STACK_SIZE,
            fx_enabled: false,

            args: Rc::new(UnsafeCell::new(Vec::new())),
            cwd: Rc::new(UnsafeCell::new(String::new())),
            memory: Rc::new(UnsafeCell::new(Vec::new())),
            files: Rc::new(UnsafeCell::new(Vec::new())),
        };

        ret.regs.sp = stack + CONTEXT_STACK_SIZE;

        for arg in args.iter() {
            ret.push(*arg);
        }

        ret.push(call); //We will ret into this function call

        ret.push(1 << 9); //Flags

        ret
    }

    #[cfg(target_arch = "x86_64")]
    pub unsafe fn new(call: usize, args: &Vec<usize>) -> Box<Self> {
        let stack = memory::alloc(CONTEXT_STACK_SIZE + 512);

        let mut ret = box Context {
            interrupted: false,
            exited: false,

            regs: Regs::default(),
            stack: stack,
            fx: stack + CONTEXT_STACK_SIZE,
            fx_enabled: false,

            args: Rc::new(UnsafeCell::new(Vec::new())),
            cwd: Rc::new(UnsafeCell::new(String::new())),
            memory: Rc::new(UnsafeCell::new(Vec::new())),
            files: Rc::new(UnsafeCell::new(Vec::new())),
        };

        let mut args_mut = args.clone();

        while args_mut.len() >= 7 {
            if let Some(value) = args_mut.pop() {
                ret.push(value);
            }
        }

        //First six args are in regs
        ret.regs.r9 = if args_mut.len() >= 6 { if let Some(value) = args_mut.pop() { value } else { 0 } } else { 0 };
        ret.regs.r8 = if args_mut.len() >= 5 { if let Some(value) = args_mut.pop() { value } else { 0 } } else { 0 };
        ret.regs.cx = if args_mut.len() >= 4 { if let Some(value) = args_mut.pop() { value } else { 0 } } else { 0 };
        ret.regs.dx = if args_mut.len() >= 3 { if let Some(value) = args_mut.pop() { value } else { 0 } } else { 0 };
        ret.regs.si = if args_mut.len() >= 2 { if let Some(value) = args_mut.pop() { value } else { 0 } } else { 0 };
        ret.regs.di = if args_mut.len() >= 1 { if let Some(value) = args_mut.pop() { value } else { 0 } } else { 0 };

        ret.push(call); //We will ret into this function call

        ret.push(1 << 9); //Flags

        ret
    }

    pub fn spawn(box_fn: Box<FnBox()>) {
        unsafe {
            let box_fn_ptr: *mut Box<FnBox()> = memory::alloc_type();
            ptr::write(box_fn_ptr, box_fn);

            let mut context_box_args: Vec<usize> = Vec::new();
            context_box_args.push(box_fn_ptr as usize);
            context_box_args.push(context_exit as usize);

            let reenable = scheduler::start_no_ints();
            if contexts_ptr as usize > 0 {
                (*contexts_ptr).push(Context::new(context_box as usize, &context_box_args));
            }
            scheduler::end_no_ints(reenable);
        }
    }

    pub unsafe fn current_i() -> usize {
        return context_i;
    }

    pub unsafe fn current<'a>() -> Option<&'a Box<Context>> {
        if context_enabled && context_i > 1 {
            let contexts = &mut *contexts_ptr;
            contexts.get(context_i)
        }else{
            None
        }
    }

    pub unsafe fn current_mut<'a>() -> Option<&'a mut Box<Context>> {
        if context_enabled && context_i > 1 {
            let contexts = &mut *contexts_ptr;
            contexts.get_mut(context_i)
        }else{
            None
        }
    }

    pub unsafe fn canonicalize(&self, path: &str) -> String {
        if path.find(':').is_none() {
            if path.starts_with('/') {
                let i = (*self.cwd.get()).find(':').unwrap_or(0) + 1;
                (*self.cwd.get())[.. i].to_string() + &path
            } else {
                (*self.cwd.get()).clone() + &path
            }
        }else{
            path.to_string()
        }
    }

    pub unsafe fn get_file<'a>(&self, fd: usize) -> Option<&'a Box<Resource>> {
        for file in (*self.files.get()).iter() {
            if file.fd == fd {
                return Some(& file.resource);
            }
        }

        None
    }

    pub unsafe fn get_file_mut<'a>(&mut self, fd: usize) -> Option<&'a mut Box<Resource>> {
        for file in (*self.files.get()).iter_mut() {
            if file.fd == fd {
                return Some(&mut file.resource);
            }
        }

        None
    }

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

    pub unsafe fn push(&mut self, data: usize) {
        self.regs.sp -= mem::size_of::<usize>();
        ptr::write(self.regs.sp as *mut usize, data);
    }

    pub unsafe fn map(&mut self) {
        for entry in (*self.memory.get()).iter_mut() {
            entry.map();
        }
    }

    pub unsafe fn unmap(&mut self) {
        for entry in (*self.memory.get()).iter_mut() {
            entry.unmap();
        }
    }

    #[cold]
    #[inline(never)]
    pub unsafe fn switch_fx(&mut self, other: &mut Self) {
        asm!("fxsave [$0]"
            :
            : "r"(self.fx)
            : "memory"
            : "intel", "volatile");

        self.fx_enabled = true;

        if other.fx_enabled {
            asm!("fxrstor [$0]"
                :
                : "r"(other.fx)
                : "memory"
                : "intel", "volatile");
        }
    }

    //Warning: This function MUST be inspected in disassembly for correct push/pop
    //It should have exactly no extra pushes or pops
    #[cold]
    #[inline(never)]
    #[cfg(target_arch = "x86")]
    pub unsafe fn switch_stack(&mut self, other: &mut Self) {
        asm!("pushfd"
            : "={esp}"(self.regs.sp)
            :
            : "memory"
            : "intel", "volatile");

        asm!("popfd"
            :
            : "{esp}"(other.regs.sp)
            : "memory"
            : "intel", "volatile");
    }

    //Warning: This function MUST be inspected in disassembly for correct push/pop
    //It should have no extra pushes or pops
    #[cold]
    #[inline(never)]
    #[cfg(target_arch = "x86_64")]
    pub unsafe fn switch_stack(&mut self, other: &mut Self) {
        asm!("pushfq"
            : "={rsp}"(self.regs.sp)
            :
            : "memory"
            : "intel", "volatile");

        asm!("popfq"
            :
            : "{rsp}"(other.regs.sp)
            : "memory"
            : "intel", "volatile");
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        if self.stack > 0 {
            unsafe { memory::unalloc(self.stack) };
        }
    }
}
