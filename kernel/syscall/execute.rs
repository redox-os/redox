//! System calls for execution of programs or threads.

use alloc::arc::Arc;

use arch::context::{CONTEXT_IMAGE_ADDR, CONTEXT_IMAGE_SIZE, CONTEXT_HEAP_ADDR, CONTEXT_HEAP_SIZE,
                    CONTEXT_MMAP_ADDR, CONTEXT_MMAP_SIZE, CONTEXT_STACK_SIZE, CONTEXT_STACK_ADDR,
                    context_switch, context_userspace, Context, ContextMemory, ContextZone};
use arch::elf::Elf;
use arch::memory;
use arch::regs::Regs;

use collections::borrow::ToOwned;
use collections::string::String;
use collections::vec::Vec;

use common::slice::GetSlice;

use core::cell::UnsafeCell;
use core::ops::DerefMut;
use core::{mem, ptr, slice, str};

use fs::Url;

use system::error::{Error, Result, ENOEXEC, ENOMEM};

pub fn execute_thread(context_ptr: *mut Context, entry: usize, mut args: Vec<String>) -> ! {
    Context::spawn("kexec".into(),
                   box move || {
        let context = unsafe { &mut *context_ptr };

        let mut context_args: Vec<usize> = Vec::new();
        context_args.push(0); // ENVP
        context_args.push(0); // ARGV NULL
        let mut argc = 0;
        while let Some(mut arg) = args.pop() {
            if ! arg.ends_with('\0') {
                arg.push('\0');
            }

            let mut physical_address = arg.as_ptr() as usize;
            if physical_address >= 0x80000000 {
                physical_address -= 0x80000000;
            }

            let virtual_address = unsafe { (*context.image.get()).next_mem() };
            let virtual_size = arg.len();

            mem::forget(arg);

            unsafe {
                (*context.image.get()).memory.push(ContextMemory {
                    physical_address: physical_address,
                    virtual_address: virtual_address,
                    virtual_size: virtual_size,
                    writeable: false,
                    allocated: true,
                });
            }

            context_args.push(virtual_address as usize);
            argc += 1;
        }
        context_args.push(argc);

        context.iopl = 0;

        context.regs = Regs::default();
        context.regs.sp = context.kernel_stack + CONTEXT_STACK_SIZE - 128;

        context.stack = Some(ContextMemory {
            physical_address: unsafe { memory::alloc_aligned(CONTEXT_STACK_SIZE, 4096) },
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

        if let Some(vfork) = context.vfork.take() {
            unsafe { (*vfork).blocked = false; }
        }
    });

    loop {
        unsafe { context_switch() };
    }
}

/// Execute an executable
pub fn execute(mut args: Vec<String>) -> Result<usize> {
    let contexts = unsafe { & *::env().contexts.get() };
    let current = try!(contexts.current());

    let mut vec: Vec<u8> = Vec::new();

    let path = current.canonicalize(args.get(0).map_or("", |p| &p));
    let url = try!(Url::from_str(&path));
    {
        let mut resource = try!(url.open());

        // Hack to allow file scheme to find memory in context's memory space
        unsafe {
            let heap = &mut *current.heap.get();

            let virtual_size = 1024*1024;
            let virtual_address = heap.next_mem();

            let physical_address = memory::alloc_aligned(virtual_size, 4096);
            if physical_address == 0 {
                return Err(Error::new(ENOMEM));
            }

            let mut memory = ContextMemory {
                physical_address: physical_address,
                virtual_address: virtual_address,
                virtual_size: virtual_size,
                writeable: true,
                allocated: true,
            };

            memory.map();

            heap.memory.push(memory);

            'reading: loop {
                let mut bytes = slice::from_raw_parts_mut(virtual_address as *mut u8, virtual_size);
                match resource.read(&mut bytes) {
                    Ok(0) => break 'reading,
                    Ok(count) => vec.extend_from_slice(bytes.get_slice(.. count)),
                    Err(err) => return Err(err)
                }
            }

            let mut memory = heap.memory.pop().unwrap();

            memory.unmap();
        }
    }

    if vec.starts_with(b"#!") {
        if let Some(mut arg) = args.get_mut(0) {
            *arg = url.to_string();
        }

        let line = unsafe { str::from_utf8_unchecked(&vec[2..]) }.lines().next().unwrap_or("");
        let mut i = 0;
        for arg in line.trim().split(' ') {
            if !arg.is_empty() {
                args.insert(i, arg.to_owned());
                i += 1;
            }
        }
        if i == 0 {
            args.insert(i, "/bin/sh".to_owned());
        }
        execute(args)
    } else {
        match Elf::from(&vec) {
            Ok(executable) => {
                let entry = unsafe { executable.entry() };
                let mut memory = Vec::new();
                unsafe {
                    for segment in executable.load_segment().iter() {
                        let virtual_address = segment.vaddr as usize;
                        let virtual_size = segment.mem_len as usize;

                        let offset = virtual_address % 4096;

                        let physical_address = memory::alloc_aligned(virtual_size + offset, 4096);

                        if physical_address > 0 {
                            //TODO: Use paging to fix collisions
                            // Copy progbits
                            debugln!("Copy to {:X}", physical_address);
                            ::memcpy((physical_address + offset) as *mut u8,
                                     (executable.data.as_ptr() as usize + segment.off as usize) as *const u8,
                                     segment.file_len as usize);

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
                    let contexts = unsafe { &mut *::env().contexts.get() };
                    let mut context = try!(contexts.current_mut());

                    //debugln!("{}: {}: execute {}", context.pid, context.name, url.string);

                    context.name = url.to_string().into();
                    context.cwd =
                        Arc::new(UnsafeCell::new(unsafe { (*context.cwd.get()).clone() }));

                    unsafe { context.unmap() };

                    let mut image = ContextZone::new(CONTEXT_IMAGE_ADDR, CONTEXT_IMAGE_SIZE);
                    image.memory = memory;

                    context.image = Arc::new(UnsafeCell::new(image));
                    context.heap = Arc::new(UnsafeCell::new(ContextZone::new(CONTEXT_HEAP_ADDR, CONTEXT_HEAP_SIZE)));
                    context.mmap = Arc::new(UnsafeCell::new(ContextZone::new(CONTEXT_MMAP_ADDR, CONTEXT_MMAP_SIZE)));
                    context.env_vars = Arc::new(UnsafeCell::new(unsafe { (*context.env_vars.get()).clone() }));

                    unsafe { context.map() };

                    execute_thread(context.deref_mut(), entry, args);
                } else {
                    Err(Error::new(ENOEXEC))
                }
            },
            Err(msg) => {
                debugln!("execute: failed to exec '{:?}': {}", url, msg);
                Err(Error::new(ENOEXEC))
            }
        }
    }
}
