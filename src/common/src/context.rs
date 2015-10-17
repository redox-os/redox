use alloc::boxed::*;

use core::mem;
use core::ptr;

use common::memory;
use common::paging::*;
use common::resource::*;
use common::scheduler;
use common::string::*;
use common::vec::*;

pub const CONTEXT_STACK_SIZE: usize = 1024 * 1024;

pub static mut contexts_ptr: *mut Vec<Context> = 0 as *mut Vec<Context>;
pub static mut context_i: usize = 0;
pub static mut context_enabled: bool = false;

/// Switch context
pub unsafe fn context_switch(interrupted: bool) {
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
            match contexts.get(current_i) {
                Some(current) => match contexts.get(context_i) {
                    Some(next) => {
                        current.interrupted = interrupted;
                        next.interrupted = false;
                        current.remap(next);
                        current.switch(next);
                    }
                    None => (),
                },
                None => (),
            }
        }
    }

    scheduler::end_no_ints(reenable);
}

pub unsafe extern "cdecl" fn context_fork(parent_i: usize){
    let reenable = scheduler::start_no_ints();

    let contexts = &mut *contexts_ptr;
    let mut context_option: Option<Context> = None;
    if let Some(parent) = contexts.get(parent_i) {
        let stack = memory::alloc(CONTEXT_STACK_SIZE + 512);
        if stack > 0 {
            ::memcpy(stack as *mut u8, parent.stack as *const u8, CONTEXT_STACK_SIZE + 512);

            let mut mem: Vec<ContextMemory> = Vec::new();
            for entry in parent.memory.iter() {
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

            let mut files: Vec<ContextFile> = Vec::new();
            for file in parent.files.iter() {
                if let Some(resource) = file.resource.dup() {
                    files.push(ContextFile {
                        fd: file.fd,
                        resource: resource
                    });
                }
            }

            context_option = Some(Context {
                stack: stack,
                stack_ptr: (parent.stack_ptr - parent.stack) + stack,
                fx: stack + CONTEXT_STACK_SIZE,
                fx_enabled: parent.fx_enabled,
                memory: mem,
                cwd: parent.cwd.clone(),
                files: files,
                interrupted: parent.interrupted,
                exited: parent.exited,
            });
        }
    }

    if let Some(context) = context_option {
        contexts.push(context);
    }

    scheduler::end_no_ints(reenable);
}

//TODO: To clean up memory leak, current must be destroyed!
pub unsafe extern "cdecl" fn context_exit() {
    let reenable = scheduler::start_no_ints();

    let contexts = &*contexts_ptr;
    if context_enabled && context_i > 1 {
        match contexts.get(context_i) {
            Some(mut current) => current.exited = true,
            None => (),
        }
    }

    scheduler::end_no_ints(reenable);

    context_switch(false);
}

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

pub struct ContextFile {
    pub fd: usize,
    pub resource: Box<Resource>,
}

pub struct Context {
    pub stack: usize,
    pub stack_ptr: usize,
    pub fx: usize,
    pub fx_enabled: bool,
    pub memory: Vec<ContextMemory>,
    pub cwd: String,
    pub files: Vec<ContextFile>,
    pub interrupted: bool,
    pub exited: bool,
}

impl Context {
    pub unsafe fn root() -> Self {
        Context {
            stack: 0,
            stack_ptr: 0,
            fx: memory::alloc(512),
            fx_enabled: false,
            memory: Vec::new(),
            cwd: String::new(),
            files: Vec::new(),
            interrupted: false,
            exited: false,
        }
    }

    #[cfg(target_arch = "x86")]
    pub unsafe fn new(call: usize, args: &Vec<usize>) -> Self {
        let stack = memory::alloc(CONTEXT_STACK_SIZE + 512);

        let mut ret = Context {
            stack: stack,
            stack_ptr: stack + CONTEXT_STACK_SIZE,
            fx: stack + CONTEXT_STACK_SIZE,
            fx_enabled: false,
            memory: Vec::new(),
            cwd: String::new(),
            files: Vec::new(),
            interrupted: false,
            exited: false,
        };

        let ebp = ret.stack_ptr;

        for arg in args.iter() {
            ret.push(*arg);
        }

        ret.push(call); //We will ret into this function call

        ret.push(0); //ESI is a param used in the switch function

        ret.push(1 << 9); //Flags

        let esp = ret.stack_ptr;

        ret.push(0); //EAX
        ret.push(0); //ECX
        ret.push(0); //EDX
        ret.push(0); //EBX
        ret.push(esp); //ESP (ignored)
        ret.push(ebp); //EBP
        ret.push(0); //ESI
        ret.push(0); //EDI

        ret
    }

    #[cfg(target_arch = "x86_64")]
    pub unsafe fn new(call: usize, args: &Vec<usize>) -> Self {
        let stack = memory::alloc(CONTEXT_STACK_SIZE + 512);

        let mut ret = Context {
            stack: stack,
            stack_ptr: stack + CONTEXT_STACK_SIZE,
            fx: stack + CONTEXT_STACK_SIZE,
            fx_enabled: false,
            memory: Vec::new(),
            cwd: String::new(),
            files: Vec::new(),
            interrupted: false,
            exited: false,
        };

        let ebp = ret.stack_ptr;

        for arg in args.iter() {
            ret.push(*arg);
        }

        ret.push(call); //We will ret into this function call

        ret.push(1 << 9); //Flags

        let esp = ret.stack_ptr;

        ret.push(0); //RAX
        ret.push(0); //RCX
        ret.push(0); //RDX
        ret.push(0); //RBX
        ret.push(ebp); //RBP
        ret.push(0); //RSI
        ret.push(0); //RDI
        ret.push(0); //R8
        ret.push(0); //R9
        ret.push(0); //R10
        ret.push(0); //R11
        ret.push(0); //R12
        ret.push(0); //R13
        ret.push(0); //R14
        ret.push(0); //R15

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

    pub unsafe fn push(&mut self, data: usize) {
        self.stack_ptr -= mem::size_of::<usize>();
        ptr::write(self.stack_ptr as *mut usize, data);
    }

    pub unsafe fn map(&mut self) {
        for entry in self.memory.iter() {
            for i in 0..(entry.virtual_size + 4095) / 4096 {
                Page::new(entry.virtual_address + i * 4096).map(entry.physical_address + i * 4096);
            }
        }
    }

    pub unsafe fn unmap(&mut self) {
        for entry in self.memory.iter() {
            for i in 0..(entry.virtual_size + 4095) / 4096 {
                Page::new(entry.virtual_address + i * 4096).map_identity();
            }
        }
    }

    pub unsafe fn remap(&mut self, other: &mut Self) {
        self.unmap();
        other.map();
    }

    //Warning: This function MUST be inspected in disassembly for correct push/pop
    //It should have exactly one extra push/pop of ESI
    #[cold]
    #[inline(never)]
    #[cfg(target_arch = "x86")]
    pub unsafe fn switch(&mut self, other: &mut Self) {
        asm!("pushfd
            pushad
            mov [esi], esp"
            :
            : "{esi}"(&mut self.stack_ptr)
            : "memory"
            : "intel", "volatile");

        asm!("fxsave [esi]"
            :
            : "{esi}"(self.fx)
            : "memory"
            : "intel", "volatile");
        self.fx_enabled = true;

        //TODO: Clear registers
        if other.fx_enabled {
            asm!("fxrstor [esi]"
                :
                : "{esi}"(other.fx)
                : "memory"
                : "intel", "volatile");
        }

        asm!("mov esp, [esi]
            popad
            popfd"
            :
            : "{esi}"(&mut other.stack_ptr)
            : "memory"
            : "intel", "volatile");
    }

    //Warning: This function MUST be inspected in disassembly for correct push/pop
    //It should no extra pushes or pops
    #[cold]
    #[inline(never)]
    #[cfg(target_arch = "x86_64")]
    pub unsafe fn switch(&mut self, other: &mut Self) {
        asm!("pushfq
            push rax
            push rcx
            push rdx
            push rbx
            push rbp
            push rsi
            push rdi
            push r8
            push r9
            push r10
            push r11
            push r12
            push r13
            push r14
            push r15
            mov [rsi], rsp"
            :
            : "{rsi}"(&mut self.stack_ptr)
            : "memory"
            : "intel", "volatile");

        asm!("fxsave [rsi]"
            :
            : "{rsi}"(self.fx)
            : "memory"
            : "intel", "volatile");
        self.fx_enabled = true;

        //TODO: Clear registers
        if other.fx_enabled {
            asm!("fxrstor [rsi]"
                :
                : "{rsi}"(other.fx)
                : "memory"
                : "intel", "volatile");
        }

        asm!("mov rsp, [rsi]
            pop r15
            pop r14
            pop r13
            pop r12
            pop r11
            pop r10
            pop r9
            pop r8
            pop rdi
            pop rsi
            pop rbp
            pop rbx
            pop rdx
            pop rcx
            pop rax
            popfq"
            :
            : "{rsi}"(&mut other.stack_ptr)
            : "memory"
            : "intel", "volatile");
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        while let Some(file) = self.files.remove(0) {
            drop(file);
        }

        while let Some(entry) = self.memory.remove(0) {
            unsafe {
                memory::unalloc(entry.physical_address);
            }
        }

        if self.stack > 0 {
            unsafe {
                memory::unalloc(self.stack);
            }
            self.stack = 0;
        }
    }
}
