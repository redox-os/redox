use collections::string::{String, ToString};
use collections::vec::Vec;

use core::ops::DerefMut;
use core::{mem, ptr};

use common::debug;
use common::elf::Elf;
use common::memory;

use scheduler;
use scheduler::context::{CONTEXT_STACK_SIZE, CONTEXT_STACK_ADDR, context_switch, context_userspace, contexts_ptr, Context, ContextFile, ContextMemory};

use schemes::Url;

/// Excecute an excecutable
// TODO: Modify current context, take current stdio
pub fn execute(url: Url, mut args: Vec<String>) {
    unsafe {
        let reenable = scheduler::start_no_ints();

        let context_ptr: *mut Context = if let Some(mut current) = Context::current_mut() {
            current.deref_mut()
        } else {
            0 as *mut Context
        };

        scheduler::end_no_ints(reenable);

        if context_ptr as usize > 0 {
            Context::spawn("kexec ".to_string() + &url.string, box move || {
                let mut memory = Vec::new();
                let mut entry = 0;

                if let Some(mut resource) = url.open() {
                    let mut vec: Vec<u8> = Vec::new();
                    resource.read_to_end(&mut vec);

                    let executable = Elf::from_data(vec.as_ptr() as usize);
                    entry = executable.entry();
                    for segment in executable.load_segment().iter() {
                        let virtual_address = segment.vaddr as usize;
                        let virtual_size = segment.mem_len as usize;
                        let physical_address = memory::alloc(virtual_size);

                        if physical_address > 0 {
                            // Copy progbits
                            ::memcpy(physical_address as *mut u8,
                                     (executable.data + segment.off as usize) as *const u8,
                                     segment.file_len as usize);
                            // Zero bss
                            ::memset((physical_address + segment.file_len as usize) as *mut u8,
                                     0,
                                     segment.mem_len as usize - segment.file_len as usize);

                             memory.push(ContextMemory {
                                 physical_address: physical_address,
                                 virtual_address: virtual_address,
                                 virtual_size: virtual_size,
                                 writeable: segment.flags & 2 == 2
                             });
                        }
                    }
                } else {
                    debug::d("Failed to open\n");
                }

                if ! memory.is_empty() && entry > 0 {
                    args.insert(0, url.to_string());

                    let mut context_args: Vec<usize> = Vec::new();
                    context_args.push(0); // ENVP
                    context_args.push(0); // ARGV NULL
                    let mut argc = 0;
                    for i in 0..args.len() {
                        if let Some(arg) = args.get(args.len() - i - 1) {
                            context_args.push(arg.as_ptr() as usize);
                            argc += 1;
                        }
                    }
                    context_args.push(argc);

                    let reenable = scheduler::start_no_ints();

                    let context = &mut * context_ptr;

                    context.name = url.to_string();

                    context.sp = context.kernel_stack + CONTEXT_STACK_SIZE - 128;

                    context.stack = Some(ContextMemory {
                        physical_address: memory::alloc(CONTEXT_STACK_SIZE),
                        virtual_address: CONTEXT_STACK_ADDR,
                        virtual_size: CONTEXT_STACK_SIZE,
                        writeable: true
                    });

                    *context.args.get() = args;
                    *context.memory.get() = memory;

                    let user_sp = if let Some(ref stack) = context.stack {
                        let mut sp = stack.physical_address + stack.virtual_size - 128;
                        for arg in context_args.iter() {
                            sp -= mem::size_of::<usize>();
                            ptr::write(sp as *mut usize, *arg);
                        }
                        sp - stack.physical_address + stack.virtual_address
                    } else {
                        0
                    };

                    context.push(0x20 | 3);
                    context.push(user_sp);
                    context.push(1 << 9);
                    context.push(0x18 | 3);
                    context.push(entry);
                    context.push(context_userspace as usize);

                    scheduler::end_no_ints(reenable);
                } else {
                    debug::d("Invalid memory or entry\n");
                }
            });

            loop {
                context_switch(false);
            }
        }
    }
}
