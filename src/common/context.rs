use alloc::boxed::*;

use core::ptr;

use common::memory;
use common::paging::*;
use common::resource::*;
use common::scheduler::*;
use common::string::*;
use common::vec::*;

pub const CONTEXT_STACK_SIZE: usize = 1024 * 1024;

pub static mut contexts_ptr: *mut Vec<Box<Context>> = 0 as *mut Vec<Box<Context>>;
pub static mut context_i: usize = 0;
pub static mut context_enabled: bool = false;

/// Switch context
pub unsafe fn context_switch(interrupted: bool) {
    let reenable = start_no_ints();

    let contexts = &mut *contexts_ptr;
    if context_enabled {
        let current_i = context_i;
        context_i += 1;
        //The only garbage collection in Redox
        loop {
            if context_i >= contexts.len() {
                context_i -= contexts.len();
            }

            let mut remove = false;
            if let Option::Some(next) = contexts.get(context_i) {
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
                Option::Some(current) => match contexts.get(context_i) {
                    Option::Some(next) => {
                        current.interrupted = interrupted;
                        next.interrupted = false;
                        current.remap(next);
                        current.switch(next);
                    }
                    Option::None => (),
                },
                Option::None => (),
            }
        }
    }

    end_no_ints(reenable);
}

//TODO: To clean up memory leak, current must be destroyed!
pub unsafe extern "cdecl" fn context_exit() {
    let reenable = start_no_ints();

    let contexts = &*contexts_ptr;
    if context_enabled && context_i > 1 {
        match contexts.get(context_i) {
            Option::Some(mut current) => current.exited = true,
            Option::None => (),
        }
    }

    end_no_ints(reenable);

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
    pub stack_ptr: u32,
    pub fx: usize,
    pub fx_enabled: bool,
    pub memory: Vec<ContextMemory>,
    pub cwd: String,
    pub files: Vec<ContextFile>,
    pub interrupted: bool,
    pub exited: bool,
}

impl Context {
    pub unsafe fn root() -> Box<Context> {
        box Context {
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

    pub unsafe fn new(call: u32, args: &Vec<u32>) -> Box<Context> {
        let stack = memory::alloc(CONTEXT_STACK_SIZE + 512);

        let mut ret = box Context {
            stack: stack,
            stack_ptr: (stack + CONTEXT_STACK_SIZE) as u32,
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

    pub fn spawn(box_fn: Box<FnBox()>) {
        unsafe {
            let box_fn_ptr: *mut Box<FnBox()> = memory::alloc_type();
            ptr::write(box_fn_ptr, box_fn);

            let mut context_box_args: Vec<u32> = Vec::new();
            context_box_args.push(box_fn_ptr as u32);
            context_box_args.push(context_exit as u32);

            let reenable = start_no_ints();
            if contexts_ptr as usize > 0 {
                (*contexts_ptr).push(Context::new(context_box as u32, &context_box_args));
            }
            end_no_ints(reenable);
        }
    }

    pub unsafe fn push(&mut self, data: u32) {
        self.stack_ptr -= 4;
        ptr::write(self.stack_ptr as *mut u32, data);
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

    pub unsafe fn remap(&mut self, other: &mut Context) {
        self.unmap();
        other.map();
    }

    //Warning: This function MUST be inspected in disassembly for correct push/pop
    //It should have exactly one extra push/pop of ESI
    #[cold]
    #[inline(never)]
    pub unsafe fn switch(&mut self, other: &mut Context) {
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
}

impl Drop for Context {
    fn drop(&mut self) {
        while let Option::Some(file) = self.files.remove(0) {
            drop(file);
        }

        while let Option::Some(entry) = self.memory.remove(0) {
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
