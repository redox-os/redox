//! System calls for execution of programs or threads.

use alloc::arc::Arc;

use arch::context::{CONTEXT_IMAGE_ADDR, CONTEXT_IMAGE_SIZE, CONTEXT_HEAP_ADDR, CONTEXT_HEAP_SIZE,
                    CONTEXT_MMAP_ADDR, CONTEXT_MMAP_SIZE, CONTEXT_STACK_SIZE, CONTEXT_STACK_ADDR,
                    context_switch, context_userspace, Context, ContextMemory, ContextZone};
use arch::elf::Elf;
use arch::memory;
use arch::regs::Regs;

use collections::borrow::ToOwned;
use collections::string::{String, ToString};
use collections::vec::Vec;

use common::slice::GetSlice;

use core::cell::UnsafeCell;
use core::ops::DerefMut;
use core::{mem, ptr, slice, str};

use system::error::{Error, Result, ENOEXEC, ENOMEM};
use system::syscall::O_RDONLY;

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

            let virtual_size = arg.len();
            let virtual_address = unsafe { (*context.image.get()).add_mem(physical_address, virtual_size, false, true) }.unwrap();

            mem::forget(arg);

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
            unsafe { (*vfork).unblock("execute_thread vfork") }
        }
    });

    loop {
        unsafe { context_switch() };
    }
}

/// Execute an executable
pub fn execute(mut args: Vec<String>) -> Result<usize> {
    let contexts = unsafe { &mut *::env().contexts.get() };
    let current = try!(contexts.current_mut());

    let mut vec: Vec<u8> = Vec::new();

    let path = current.canonicalize(args.get(0).map_or("", |p| &p));
    {
        let mut resource = try!(::env().open(&path, O_RDONLY));

        // Hack to allow file scheme to find memory in context's memory space
        unsafe {
            let mmap = &mut *current.mmap.get();

            let virtual_size = 1024*1024;

            let physical_address = memory::alloc_aligned(virtual_size, 4096);
            if physical_address == 0 {
                return Err(Error::new(ENOMEM));
            }

            let virtual_address = try!(mmap.add_mem(physical_address, virtual_size, true, true));

            for i in 0..mmap.memory.len() {
                if mmap.memory[i].virtual_address == virtual_address {
                    mmap.memory[i].map();
                    break;
                }
            }

            let mut read_loop = || -> Result<usize> {
                loop {
                    let mut bytes = slice::from_raw_parts_mut(virtual_address as *mut u8, virtual_size);
                    match resource.read(&mut bytes) {
                        Ok(0) => return Ok(0),
                        Ok(count) => vec.extend_from_slice(bytes.get_slice(.. count)),
                        Err(err) => return Err(err)
                    }
                }
            };

            let res = read_loop();

            for i in 0..mmap.memory.len() {
                if mmap.memory[i].virtual_address == virtual_address {
                    mmap.memory.remove(i).unmap();
                    break;
                }
            }

            try!(res);
        }
    }

    if vec.starts_with(b"#!") {
        if let Some(mut arg) = args.get_mut(0) {
            *arg = path.to_string();
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
                let segments = unsafe { executable.load_segment() };

                if entry > 0 && ! segments.is_empty() {
                    unsafe { current.unmap() };

                    current.name = path.to_string().into();
                    current.cwd = Arc::new(UnsafeCell::new(unsafe { (*current.cwd.get()).clone() }));

                    current.image = Arc::new(UnsafeCell::new(ContextZone::new(CONTEXT_IMAGE_ADDR, CONTEXT_IMAGE_SIZE)));
                    current.heap = Arc::new(UnsafeCell::new(ContextZone::new(CONTEXT_HEAP_ADDR, CONTEXT_HEAP_SIZE)));
                    current.mmap = Arc::new(UnsafeCell::new(ContextZone::new(CONTEXT_MMAP_ADDR, CONTEXT_MMAP_SIZE)));
                    current.env_vars = Arc::new(UnsafeCell::new(unsafe { (*current.env_vars.get()).clone() }));

                    {
                        let image = unsafe { &mut *current.image.get() };

                        for segment in segments.iter() {
                            let virtual_address = segment.vaddr as usize;
                            let virtual_size = segment.mem_len as usize;

                            let offset = virtual_address % 4096;

                            let physical_address = unsafe { memory::alloc_aligned(virtual_size + offset, 4096) };

                            if physical_address == 0 {
                                panic!("OOM in exec");
                            }

                            let mut memory = ContextMemory {
                                physical_address: physical_address,
                                virtual_address: virtual_address - offset,
                                virtual_size: virtual_size + offset,
                                writeable: true,
                                allocated: true,
                            };

                            unsafe { memory.map() };

                            // Copy progbits
                            unsafe {
                                ::memcpy(virtual_address as *mut u8,
                                        executable.data.as_ptr().offset(segment.off as isize),
                                        segment.file_len as usize)
                            };

                            unsafe { memory.unmap() };

                            memory.writeable = segment.flags & 2 == 2;

                            image.memory.push(memory);
                        }
                    }

                    //debugln!("{}: {}: execute {}", context.pid, context.name, url.string);

                    unsafe { current.map() };

                    execute_thread(current.deref_mut(), entry, args);
                } else {
                    Err(Error::new(ENOEXEC))
                }
            },
            Err(msg) => {
                debugln!("execute: failed to exec '{:?}': {}", path, msg);
                Err(Error::new(ENOEXEC))
            }
        }
    }
}
