use alloc::arc::Arc;

use arch::context::{CONTEXT_STACK_SIZE, CONTEXT_STACK_ADDR, context_switch, context_userspace, Context, ContextMemory};
use arch::elf::Elf;
use arch::memory;

use collections::string::{String, ToString};
use collections::vec::Vec;

use common::slice::GetSlice;

use core::cell::UnsafeCell;
use core::ops::DerefMut;
use core::{mem, ptr};

use fs::Url;

use system::error::{Error, Result, ESRCH, ENOEXEC};

fn execute_inner(url: &Url, args: &Vec<String>) -> Result<(*mut Context, usize)> {
    let mut resource = try!(url.open());

    let mut vec: Vec<u8> = Vec::new();
    'reading: loop {
        let mut bytes = [0; 4096];
        match resource.read(&mut bytes) {
            Ok(0) => break 'reading,
            Ok(count) => vec.push_all(bytes.get_slice(.. count)),
            Err(err) => return Err(err)
        }
    }

    match Elf::from(&vec) {
        Ok(executable) => {
            let entry = unsafe { executable.entry() };
            let mut memory = Vec::new();
            unsafe {
                for segment in executable.load_segment().iter() {
                    let virtual_address = segment.vaddr as usize;
                    let virtual_size = segment.mem_len as usize;

                    let offset = virtual_address % 4096;

                    let physical_address = memory::alloc(virtual_size + offset);

                    if physical_address > 0 {
                        // Copy progbits
                        ::memcpy((physical_address + offset) as *mut u8,
                                 (executable.data.as_ptr() as usize + segment.off as usize) as *const u8,
                                 segment.file_len as usize);
                        // Zero bss
                        if segment.mem_len > segment.file_len {
                            ::memset((physical_address + offset + segment.file_len as usize) as *mut u8,
                                    0,
                                    segment.mem_len as usize - segment.file_len as usize);
                        }

                        memory.push(ContextMemory {
                            physical_address: physical_address,
                            virtual_address: virtual_address - offset,
                            virtual_size: virtual_size + offset,
                            writeable: segment.flags & 2 == 2,
                            allocated: true,
                        });
                    }
                }
            }

            if entry > 0 && ! memory.is_empty() {
                let mut contexts = ::env().contexts.lock();
                if let Some(mut context) = contexts.current_mut() {
                    if let Some(arg) = args.get(0) {
                        context.name = arg.clone();
                    }
                    context.cwd = Arc::new(UnsafeCell::new(unsafe { (*context.cwd.get()).clone() }));

                    unsafe { context.unmap() };
                    context.memory = Arc::new(UnsafeCell::new(memory));
                    unsafe { context.map() };

                    Ok((context.deref_mut(), entry))
                } else {
                    Err(Error::new(ESRCH))
                }
            } else {
                Err(Error::new(ENOEXEC))
            }
        },
        Err(msg) => {
            debugln!("execute: failed to exec '{}': {}", url.string, msg);
            Err(Error::new(ENOEXEC))
        }
    }
}

pub fn execute_outer(context_ptr: *mut Context, entry: usize, mut args: Vec<String>) -> ! {
    Context::spawn("kexec".to_string(), box move || {
        let context = unsafe { &mut *context_ptr };

        let mut context_args: Vec<usize> = Vec::new();
        context_args.push(0); // ENVP
        context_args.push(0); // ARGV NULL
        let mut argc = 0;
        for i in 0..args.len() {
            let reverse_i = args.len() - i - 1;
            if let Some(ref mut arg) = args.get_mut(reverse_i) {
                if ! arg.ends_with('\0') {
                    arg.push('\0');
                }

                let physical_address = arg.as_ptr() as usize;
                let virtual_address = unsafe { context.next_mem() };
                let virtual_size = arg.len();

                mem::forget(arg);

                unsafe {
                    (*context.memory.get()).push(ContextMemory {
                        physical_address: physical_address,
                        virtual_address: virtual_address,
                        virtual_size: virtual_size,
                        //TODO: Remove this hack for brk
                        writeable: true,
                        allocated: true,
                    });
                }

                context_args.push(virtual_address as usize);
                argc += 1;
            }
        }
        context_args.push(argc);

        context.sp = context.kernel_stack + CONTEXT_STACK_SIZE - 128;

        context.stack = Some(ContextMemory {
            physical_address: unsafe { memory::alloc(CONTEXT_STACK_SIZE) },
            virtual_address: CONTEXT_STACK_ADDR,
            virtual_size: CONTEXT_STACK_SIZE,
            writeable: true,
            allocated: true,
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

/// Execute an executable
pub fn execute(args: Vec<String>) -> Result<usize> {
    let contexts = ::env().contexts.lock();
    if let Some(current) = contexts.current() {
        let path = args.get(0).map_or(String::new(), |p| p.clone());

        if let Ok((context_ptr, entry)) = execute_inner(&Url::from_string(current.canonicalize(&path)), &args) {
            execute_outer(context_ptr, entry, args);
        }else{
            let (context_ptr, entry) = try!(execute_inner(&Url::from_string("file:/bin/".to_string() + &path), &args));
            execute_outer(context_ptr, entry, args);
        }
    } else {
        Err(Error::new(ESRCH))
    }
}
