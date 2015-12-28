use alloc::arc::Arc;

use collections::string::{String, ToString};
use collections::vec::Vec;

use core::cell::UnsafeCell;
use core::ops::DerefMut;
use core::{mem, ptr};

use common::elf::Elf;
use common::memory;

use scheduler::context::{CONTEXT_STACK_SIZE, CONTEXT_STACK_ADDR, context_switch,
context_userspace, Context, ContextMemory};

use schemes::Url;

use sync::Intex;

/// Execute an executable (return true if successful)
pub fn execute(url: Url, mut args: Vec<String>) -> bool {
    let mut context_ptr: *mut Context = 0 as *mut Context;
    let mut entry: usize = 0;

    if let Ok(mut resource) = url.open() {
        let mut vec: Vec<u8> = Vec::new();
        if let Err(_) = resource.read_to_end(&mut vec) {
            debugln!("Failed to read executable, aborting execution");
            return false;
        }

        let executable = Elf::from_data(vec.as_ptr() as usize);
        entry = unsafe { executable.entry() };
        let mut memory = Vec::new();
        unsafe {
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

        if entry > 0 && ! memory.is_empty() {
            args.insert(0, url.to_string());

            let mut contexts = ::env().contexts.lock();
            if let Some(mut context) = contexts.current_mut() {
                context.name = url.string;
                context.args = Arc::new(UnsafeCell::new(args));
                context.cwd = Arc::new(UnsafeCell::new(unsafe { (*context.cwd.get()).clone() }));

                unsafe { context.unmap() };
                context.memory = Arc::new(UnsafeCell::new(memory));
                unsafe { context.map() };

                context_ptr = context.deref_mut();
            }
        } else {
            debug!("{}: Invalid memory or entry\n", url.string);
        }
    } else {
        debug!("{}: Failed to open\n", url.string);
    }

    if context_ptr as usize > 0 {
        Context::spawn("kexec".to_string(), box move || {
            let _intex = Intex::static_lock();

            let context = unsafe { &mut *context_ptr };

            let mut context_args: Vec<usize> = Vec::new();
            context_args.push(0); // ENVP
            context_args.push(0); // ARGV NULL
            let mut argc = 0;
            for i in 0..unsafe { (*context.args.get()).len() } {
                let reverse_i = unsafe { (*context.args.get()).len() } - i - 1;
                if let Some(ref mut arg) = unsafe { (*context.args.get()).get_mut(reverse_i) } {
                    if ! arg.ends_with('\0') {
                        arg.push('\0');
                    }
                    context_args.push(arg.as_ptr() as usize);
                    argc += 1;
                }
            }
            context_args.push(argc);

            context.sp = context.kernel_stack + CONTEXT_STACK_SIZE - 128;

            context.stack = Some(ContextMemory {
                physical_address: unsafe { memory::alloc(CONTEXT_STACK_SIZE) },
                virtual_address: CONTEXT_STACK_ADDR,
                virtual_size: CONTEXT_STACK_SIZE,
                writeable: true
            });

            let user_sp = if let Some(ref stack) = context.stack {
                let mut sp = stack.physical_address + stack.virtual_size - 128;
                for arg in context_args.iter() {
                    sp -= mem::size_of::<usize>();
                    unsafe { ptr::write(sp as *mut usize, *arg) };
                }
                sp - stack.physical_address + stack.virtual_address
            } else {
                0
            };

            unsafe {
                context.push(0x20 | 3);
                context.push(user_sp);
                context.push(1 << 9);
                context.push(0x18 | 3);
                context.push(entry);
                context.push(context_userspace as usize);
            }
        });

        loop {
            unsafe { context_switch(false) };
        }
    }
    false
}
