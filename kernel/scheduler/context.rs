use common::get_slice::GetSlice;

use alloc::boxed::{Box, FnBox};
use alloc::rc::Rc;

use collections::string::{String, ToString};
use collections::vec::Vec;

use core::cell::UnsafeCell;
use core::{mem, ptr};

use common::memory;
use common::paging::Page;
use scheduler;

use schemes::Resource;

use syscall::common::{CLONE_FILES, CLONE_FS, CLONE_VM, Regs};

pub const CONTEXT_STACK_SIZE: usize = 1024 * 1024;
pub const CONTEXT_STACK_ADDR: usize = 0xC0000000 - CONTEXT_STACK_SIZE;

pub static mut contexts_ptr: *mut Vec<Box<Context>> = 0 as *mut Vec<Box<Context>>;
pub static mut context_i: usize = 0;
pub static mut context_enabled: bool = false;

/// Switch context
///
/// Unsafe due to interrupt disabling, raw pointers, and unsafe Context functions
pub unsafe fn context_switch(regs: &mut Regs, interrupted: bool) {
    let reenable = scheduler::start_no_ints();

    let contexts = &mut *contexts_ptr;
    if context_enabled {
        let current_i = context_i;
        context_i += 1;

        if context_i >= contexts.len() {
            context_i -= contexts.len();
        }

        if context_i != current_i {
            if let Some(mut current) = contexts.get_mut(current_i) {
                current.interrupted = interrupted;

                current.save(regs);
                current.unmap();
            }

            if let Some(mut next) = contexts.get_mut(context_i) {
                next.interrupted = false;

                next.map();
                next.restore(regs);
            }
        }
    }

    scheduler::end_no_ints(reenable);
}

//TODO: To clean up memory leak, current must be destroyed!
/// Exit context
///
/// Unsafe due to interrupt disabling and raw pointers
pub unsafe fn context_exit(regs: &mut Regs) {
    let reenable = scheduler::start_no_ints();

    let contexts = &mut *contexts_ptr;
    if context_enabled {
        let old_i = context_i;

        context_switch(regs, false);

        contexts.remove(old_i);

        if old_i < context_i {
            context_i -= 1;
        }
    }

    scheduler::end_no_ints(reenable);
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
    /* } */

    /* These members control the stack and registers and are unique to each context { */
        /// The context registers
        pub regs: Regs,
        /// The context stack
        pub stack: usize,
        /// The location used to save and load SSE and FPU registers
        pub fx: usize,
        /// Indicates that registers can be loaded (they must be saved first)
        pub loadable: bool,
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

           regs: Regs::default(),
           stack: 0,
           fx: memory::alloc(512),
           loadable: false,

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

            regs: Regs::default(),
            stack: stack,
            fx: stack + CONTEXT_STACK_SIZE,
            loadable: false,

            args: Rc::new(UnsafeCell::new(Vec::new())),
            cwd: Rc::new(UnsafeCell::new(String::new())),
            memory: Rc::new(UnsafeCell::new(Vec::new())),
            files: Rc::new(UnsafeCell::new(Vec::new())),
        };

        ret.regs.sp_kernel = stack + CONTEXT_STACK_SIZE - 128;

        for arg in args.iter() {
            ret.push(*arg);
        }

        let sp = ret.regs.sp_kernel - stack + CONTEXT_STACK_ADDR;
        ret.push(0x20 | 3);
        ret.push(sp);
        ret.push(1 << 9);
        ret.push(0x18 | 3);
        ret.push(call);

        ret
    }

    #[cfg(target_arch = "x86_64")]
    pub unsafe fn new(call: usize, args: &Vec<usize>) -> Box<Self> {
        let stack = memory::alloc(CONTEXT_STACK_SIZE + 512);

        let mut ret = box Context {
            interrupted: false,

            regs: Regs::default(),
            stack: stack,
            fx: stack + CONTEXT_STACK_SIZE,
            loadable: false,

            args: Rc::new(UnsafeCell::new(Vec::new())),
            cwd: Rc::new(UnsafeCell::new(String::new())),
            memory: Rc::new(UnsafeCell::new(Vec::new())),
            files: Rc::new(UnsafeCell::new(Vec::new())),
        };

        ret.regs.sp_kernel = stack + CONTEXT_STACK_SIZE - 128;

        let mut args_mut = args.clone();

        while args_mut.len() >= 7 {
            ret.push(args_mut.remove(6));
        }

        //First six args are in regs
        ret.regs.r9 = if args_mut.len() >= 6 { if let Some(value) = args_mut.pop() { value } else { 0 } } else { 0 };
        ret.regs.r8 = if args_mut.len() >= 5 { if let Some(value) = args_mut.pop() { value } else { 0 } } else { 0 };
        ret.regs.cx = if args_mut.len() >= 4 { if let Some(value) = args_mut.pop() { value } else { 0 } } else { 0 };
        ret.regs.dx = if args_mut.len() >= 3 { if let Some(value) = args_mut.pop() { value } else { 0 } } else { 0 };
        ret.regs.si = if args_mut.len() >= 2 { if let Some(value) = args_mut.pop() { value } else { 0 } } else { 0 };
        ret.regs.di = if args_mut.len() >= 1 { if let Some(value) = args_mut.pop() { value } else { 0 } } else { 0 };

        let sp = ret.regs.sp_kernel - stack + CONTEXT_STACK_ADDR;
        ret.push(0x20 | 3);
        ret.push(sp);
        ret.push(1 << 9);
        ret.push(0x18 | 3);
        ret.push(call);

        ret
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
            let cwd = &*self.cwd.get();
            if path.starts_with('/') {
                cwd.get_slice(None, Some(cwd.find(':').map_or(1, |i| i + 1))).to_string() + &path
            } else {
                cwd.to_string() + &path
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
        self.regs.sp_kernel -= mem::size_of::<usize>();
        ptr::write(self.regs.sp_kernel as *mut usize, data);
    }

    pub unsafe fn map(&mut self) {
        if self.stack > 0 {
            for i in 0..(CONTEXT_STACK_SIZE + 4095) / 4096 {
                Page::new(CONTEXT_STACK_ADDR + i * 4096).map(self.stack + i * 4096);
            }
        }
        for entry in (*self.memory.get()).iter_mut() {
            entry.map();
        }
    }

    pub unsafe fn unmap(&mut self) {
        for entry in (*self.memory.get()).iter_mut() {
            entry.unmap();
        }
        if self.stack > 0 {
            for i in 0..(CONTEXT_STACK_SIZE + 4095) / 4096 {
                Page::new(CONTEXT_STACK_ADDR + i * 4096).map_identity();
            }
        }
    }

    //Warning: This function MUST be inspected in disassembly for correct push/pop
    //It should have exactly no extra pushes or pops
    #[cold]
    #[inline(never)]
    #[cfg(target_arch = "x86")]
    pub unsafe fn save(&mut self, regs: &mut Regs) {
        self.regs = *regs;

        asm!("fxsave [$0]"
            :
            : "r"(self.fx)
            : "memory"
            : "intel", "volatile");

        self.loadable = true;
    }

    //Warning: This function MUST be inspected in disassembly for correct push/pop
    //It should have exactly no extra pushes or pops
    #[cold]
    #[inline(never)]
    #[cfg(target_arch = "x86")]
    pub unsafe fn restore(&mut self, regs: &mut Regs) {
        if self.loadable {
            asm!("fxrstor [$0]"
                :
                : "r"(self.fx)
                : "memory"
                : "intel", "volatile");
        }

        //Save extra stuff
        self.regs.ip = regs.ip;
        self.regs.cs = regs.cs;
        self.regs.flags = regs.flags;
        self.regs.sp = regs.sp;
        self.regs.ss = regs.ss;

        *regs = self.regs;
    }

    //Warning: This function MUST be inspected in disassembly for correct push/pop
    //It should have exactly no extra pushes or pops
    #[cold]
    #[inline(never)]
    #[cfg(target_arch = "x86_64")]
    pub unsafe fn save(&mut self, regs: &mut Regs) {
        self.regs = *regs;

        asm!("fxsave [$0]"
            :
            : "r"(self.fx)
            : "memory"
            : "intel", "volatile");

        self.loadable = true;
    }

    //Warning: This function MUST be inspected in disassembly for correct push/pop
    //It should have exactly no extra pushes or pops
    #[cold]
    #[inline(never)]
    #[cfg(target_arch = "x86_64")]
    pub unsafe fn restore(&mut self, regs: &mut Regs) {
        if self.loadable {
            asm!("fxrstor [$0]"
                :
                : "r"(self.fx)
                : "memory"
                : "intel", "volatile");
        }

        //Save extra stuff
        self.regs.ip = regs.ip;
        self.regs.cs = regs.cs;
        self.regs.flags = regs.flags;
        self.regs.sp = regs.sp;
        self.regs.ss = regs.ss;

        *regs = self.regs;
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        if self.stack > 0 {
            unsafe { memory::unalloc(self.stack) };
        }
    }
}
