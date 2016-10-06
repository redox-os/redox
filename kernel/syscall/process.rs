///! Process syscalls
use alloc::arc::Arc;
use alloc::boxed::Box;
use collections::Vec;
use core::mem;
use core::str;
use spin::Mutex;

use arch;
use arch::externs::memcpy;
use arch::memory::{allocate_frame, allocate_frames, deallocate_frames, Frame};
use arch::paging::{ActivePageTable, InactivePageTable, Page, PhysicalAddress, VirtualAddress, entry};
use arch::paging::temporary_page::TemporaryPage;
use arch::start::usermode;
use context;
use context::memory::Grant;
use elf::{self, program_header};
use scheme;
use syscall;
use syscall::data::Stat;
use syscall::error::*;
use syscall::flag::{CLONE_VFORK, CLONE_VM, CLONE_FS, CLONE_FILES, MAP_WRITE, MAP_WRITE_COMBINE, WNOHANG};
use syscall::validate::{validate_slice, validate_slice_mut};

pub fn brk(address: usize) -> Result<usize> {
    let contexts = context::contexts();
    let context_lock = contexts.current().ok_or(Error::new(ESRCH))?;
    let context = context_lock.read();

    let current = if let Some(ref heap_shared) = context.heap {
        heap_shared.with(|heap| {
            heap.start_address().get() + heap.size()
        })
    } else {
        panic!("user heap not initialized");
    };

    if address == 0 {
        //println!("Brk query {:X}", current);
        Ok(current)
    } else if address >= arch::USER_HEAP_OFFSET {
        //TODO: out of memory errors
        if let Some(ref heap_shared) = context.heap {
            heap_shared.with(|heap| {
                heap.resize(address - arch::USER_HEAP_OFFSET, true, true);
            });
        } else {
            panic!("user heap not initialized");
        }

        Ok(address)
    } else {
        Err(Error::new(ENOMEM))
    }
}

pub fn clone(flags: usize, stack_base: usize) -> Result<usize> {
    let ppid;
    let pid;
    {
        let uid;
        let gid;
        let arch;
        let vfork;
        let mut kfx_option = None;
        let mut kstack_option = None;
        let mut offset = 0;
        let mut image = vec![];
        let mut heap_option = None;
        let mut stack_option = None;
        let grants;
        let cwd;
        let env;
        let files;

        // Copy from old process
        {
            let contexts = context::contexts();
            let context_lock = contexts.current().ok_or(Error::new(ESRCH))?;
            let context = context_lock.read();

            ppid = context.id;
            uid = context.uid;
            gid = context.gid;

            arch = context.arch.clone();

            if let Some(ref fx) = context.kfx {
                let mut new_fx = unsafe { Box::from_raw(::alloc::heap::allocate(512, 16) as *mut [u8; 512]) };
                for (new_b, b) in new_fx.iter_mut().zip(fx.iter()) {
                    *new_b = *b;
                }
                kfx_option = Some(new_fx);
            }

            if let Some(ref stack) = context.kstack {
                offset = stack_base - stack.as_ptr() as usize - mem::size_of::<usize>(); // Add clone ret
                let mut new_stack = stack.clone();

                unsafe {
                    let func_ptr = new_stack.as_mut_ptr().offset(offset as isize);
                    *(func_ptr as *mut usize) = arch::interrupt::syscall::clone_ret as usize;
                }

                kstack_option = Some(new_stack);
            }

            if flags & CLONE_VM == CLONE_VM {
                for memory_shared in context.image.iter() {
                    image.push(memory_shared.clone());
                }

                if let Some(ref heap_shared) = context.heap {
                    heap_option = Some(heap_shared.clone());
                }
            } else {
                for memory_shared in context.image.iter() {
                    memory_shared.with(|memory| {
                        let mut new_memory = context::memory::Memory::new(
                            VirtualAddress::new(memory.start_address().get() + arch::USER_TMP_OFFSET),
                            memory.size(),
                            entry::PRESENT | entry::NO_EXECUTE | entry::WRITABLE,
                            true,
                            false
                        );

                        unsafe {
                            arch::externs::memcpy(new_memory.start_address().get() as *mut u8,
                                                  memory.start_address().get() as *const u8,
                                                  memory.size());
                        }

                        new_memory.remap(memory.flags(), true);
                        image.push(new_memory.to_shared());
                    });
                }

                if let Some(ref heap_shared) = context.heap {
                    heap_shared.with(|heap| {
                        let mut new_heap = context::memory::Memory::new(
                            VirtualAddress::new(arch::USER_TMP_HEAP_OFFSET),
                            heap.size(),
                            entry::PRESENT | entry::NO_EXECUTE | entry::WRITABLE,
                            true,
                            false
                        );

                        unsafe {
                            arch::externs::memcpy(new_heap.start_address().get() as *mut u8,
                                                  heap.start_address().get() as *const u8,
                                                  heap.size());
                        }

                        new_heap.remap(heap.flags(), true);
                        heap_option = Some(new_heap.to_shared());
                    });
                }
            }

            if let Some(ref stack) = context.stack {
                let mut new_stack = context::memory::Memory::new(
                    VirtualAddress::new(arch::USER_TMP_STACK_OFFSET),
                    stack.size(),
                    entry::PRESENT | entry::NO_EXECUTE | entry::WRITABLE,
                    true,
                    false
                );

                unsafe {
                    arch::externs::memcpy(new_stack.start_address().get() as *mut u8,
                                          stack.start_address().get() as *const u8,
                                          stack.size());
                }

                new_stack.remap(stack.flags(), true);
                stack_option = Some(new_stack);
            }

            if flags & CLONE_VM == CLONE_VM {
                grants = context.grants.clone();
            } else {
                grants = Arc::new(Mutex::new(Vec::new()));
            }

            if flags & CLONE_FS == CLONE_FS {
                cwd = context.cwd.clone();
            } else {
                cwd = Arc::new(Mutex::new(context.cwd.lock().clone()));
            }

            if flags & CLONE_VM == CLONE_VM {
                env = context.env.clone();
            } else {
                env = Arc::new(Mutex::new(context.env.lock().clone()));
            }

            if flags & CLONE_FILES == CLONE_FILES {
                files = context.files.clone();
            } else {
                files = Arc::new(Mutex::new(context.files.lock().clone()));
            }
        }

        // If not cloning files, dup to get a new number from scheme
        // This has to be done outside the context lock to prevent deadlocks
        if flags & CLONE_FILES == 0 {
            for (fd, mut file_option) in files.lock().iter_mut().enumerate() {
                let new_file_option = if let Some(file) = *file_option {
                    let result = {
                        let scheme = {
                            let schemes = scheme::schemes();
                            let scheme = schemes.get(file.scheme).ok_or(Error::new(EBADF))?;
                            scheme.clone()
                        };
                        let result = scheme.dup(file.number);
                        result
                    };
                    match result {
                        Ok(new_number) => {
                            Some(context::file::File { scheme: file.scheme, number: new_number })
                        },
                        Err(err) => {
                            println!("clone: failed to dup {}: {:?}", fd, err);
                            None
                        }
                    }
                } else {
                    None
                };

                *file_option = new_file_option;
            }
        }

        // If vfork, block the current process
        // This has to be done after the operations that may require context switches
        if flags & CLONE_VFORK == CLONE_VFORK {
            let contexts = context::contexts();
            let context_lock = contexts.current().ok_or(Error::new(ESRCH))?;
            let mut context = context_lock.write();
            context.status = context::Status::Blocked;
            vfork = true;
        } else {
            vfork = false;
        }

        // Set up new process
        {
            let mut contexts = context::contexts_mut();
            let context_lock = contexts.new_context()?;
            let mut context = context_lock.write();

            pid = context.id;

            context.ppid = ppid;
            context.uid = uid;
            context.gid = gid;

            context.status = context::Status::Runnable;

            context.vfork = vfork;

            context.arch = arch;

            let mut active_table = unsafe { ActivePageTable::new() };

            let mut temporary_page = TemporaryPage::new(Page::containing_address(VirtualAddress::new(0x8_0000_0000)));

            let mut new_table = {
                let frame = allocate_frame().expect("no more frames in syscall::clone new_table");
                InactivePageTable::new(frame, &mut active_table, &mut temporary_page)
            };

            context.arch.set_page_table(unsafe { new_table.address() });

            // Copy kernel mapping
            {
                let frame = active_table.p4()[510].pointed_frame().expect("kernel table not mapped");
                let flags = active_table.p4()[510].flags();
                active_table.with(&mut new_table, &mut temporary_page, |mapper| {
                    mapper.p4_mut()[510].set(frame, flags);
                });
            }

            if let Some(fx) = kfx_option.take() {
                context.arch.set_fx(fx.as_ptr() as usize);
                context.kfx = Some(fx);
            }

            // Set kernel stack
            if let Some(stack) = kstack_option.take() {
                context.arch.set_stack(stack.as_ptr() as usize + offset);
                context.kstack = Some(stack);
            }

            // Setup heap
            if flags & CLONE_VM == CLONE_VM {
                // Copy user image mapping, if found
                if ! image.is_empty() {
                    let frame = active_table.p4()[0].pointed_frame().expect("user image not mapped");
                    let flags = active_table.p4()[0].flags();
                    active_table.with(&mut new_table, &mut temporary_page, |mapper| {
                        mapper.p4_mut()[0].set(frame, flags);
                    });
                }
                context.image = image;

                // Copy user heap mapping, if found
                if let Some(heap_shared) = heap_option {
                    let frame = active_table.p4()[1].pointed_frame().expect("user heap not mapped");
                    let flags = active_table.p4()[1].flags();
                    active_table.with(&mut new_table, &mut temporary_page, |mapper| {
                        mapper.p4_mut()[1].set(frame, flags);
                    });
                    context.heap = Some(heap_shared);
                }

                if ! grants.lock().is_empty() {
                    let frame = active_table.p4()[2].pointed_frame().expect("user heap not mapped");
                    let flags = active_table.p4()[2].flags();
                    active_table.with(&mut new_table, &mut temporary_page, |mapper| {
                        mapper.p4_mut()[1].set(frame, flags);
                    });
                }
                context.grants = grants;
            } else {
                // Copy percpu mapping
                {
                    extern {
                        /// The starting byte of the thread data segment
                        static mut __tdata_start: u8;
                        /// The ending byte of the thread BSS segment
                        static mut __tbss_end: u8;
                    }

                    let size = unsafe { & __tbss_end as *const _ as usize - & __tdata_start as *const _ as usize };

                    let start = arch::KERNEL_PERCPU_OFFSET + arch::KERNEL_PERCPU_SIZE * ::cpu_id();
                    let end = start + size;

                    let start_page = Page::containing_address(VirtualAddress::new(start));
                    let end_page = Page::containing_address(VirtualAddress::new(end - 1));
                    for page in Page::range_inclusive(start_page, end_page) {
                        let frame = active_table.translate_page(page).expect("kernel percpu not mapped");
                        active_table.with(&mut new_table, &mut temporary_page, |mapper| {
                            mapper.map_to(page, frame, entry::PRESENT | entry::NO_EXECUTE | entry::WRITABLE);
                        });
                    }
                }

                // Move copy of image
                for memory_shared in image.iter_mut() {
                    memory_shared.with(|memory| {
                        let start = VirtualAddress::new(memory.start_address().get() - arch::USER_TMP_OFFSET + arch::USER_OFFSET);
                        memory.move_to(start, &mut new_table, &mut temporary_page, true);
                    });
                }
                context.image = image;

                // Move copy of heap
                if let Some(heap_shared) = heap_option {
                    heap_shared.with(|heap| {
                        heap.move_to(VirtualAddress::new(arch::USER_HEAP_OFFSET), &mut new_table, &mut temporary_page, true);
                    });
                    context.heap = Some(heap_shared);
                }
            }

            // Setup user stack
            if let Some(mut stack) = stack_option {
                stack.move_to(VirtualAddress::new(arch::USER_STACK_OFFSET), &mut new_table, &mut temporary_page, true);
                context.stack = Some(stack);
            }

            context.cwd = cwd;

            context.env = env;

            context.files = files;
        }
    }

    unsafe { context::switch(); }

    Ok(pid)
}

pub fn exit(status: usize) -> ! {
    {
        let contexts = context::contexts();
        let (vfork, ppid) = {
            let context_lock = contexts.current().expect("tried to exit without context");
            let mut context = context_lock.write();
            context.image.clear();
            drop(context.heap.take());
            drop(context.stack.take());
            context.grants = Arc::new(Mutex::new(Vec::new()));
            context.files = Arc::new(Mutex::new(Vec::new()));
            context.status = context::Status::Exited(status);

            let vfork = context.vfork;
            context.vfork = false;
            (vfork, context.ppid)
        };
        if vfork {
            if let Some(context_lock) = contexts.get(ppid) {
                let mut context = context_lock.write();
                if context.status == context::Status::Blocked {
                    context.status = context::Status::Runnable;
                } else {
                    println!("{} not blocked for exit vfork unblock", ppid);
                }
            } else {
                println!("{} not found for exit vfork unblock", ppid);
            }
        }
    }

    unsafe { context::switch(); }

    unreachable!();
}

pub fn exec(path: &[u8], arg_ptrs: &[[usize; 2]]) -> Result<usize> {
    let entry;
    let mut sp = arch::USER_STACK_OFFSET + arch::USER_STACK_SIZE - 256;

    {
        let mut args = Vec::new();
        for arg_ptr in arg_ptrs {
            let arg = validate_slice(arg_ptr[0] as *const u8, arg_ptr[1])?;
            args.push(arg.to_vec()); // Must be moved into kernel space before exec unmaps all memory
        }

        let (uid, gid) = {
            let contexts = context::contexts();
            let context_lock = contexts.current().ok_or(Error::new(ESRCH))?;
            let context = context_lock.read();
            (context.uid, context.gid)
        };

        let file = syscall::open(path, 0)?;
        let mut stat = Stat::default();
        syscall::file_op_mut_slice(syscall::number::SYS_FSTAT, file, &mut stat)?;

        let mut perm = stat.st_mode & 0o7;
        if stat.st_uid == uid {
            perm |= (stat.st_mode >> 6) & 0o7;
        }
        if stat.st_gid == gid {
            perm |= (stat.st_mode >> 3) & 0o7;
        }
        if uid == 0 {
            perm |= 0o7;
        }

        if perm & 0o1 != 0o1 {
            let _ = syscall::close(file);
            return Err(Error::new(EACCES));
        }

        //TODO: Only read elf header, not entire file. Then read required segments
        let mut data = vec![0; stat.st_size as usize];
        syscall::file_op_mut_slice(syscall::number::SYS_READ, file, &mut data)?;
        let _ = syscall::close(file);

        match elf::Elf::from(&data) {
            Ok(elf) => {
                entry = elf.entry();

                drop(path); // Drop so that usage is not allowed after unmapping context
                drop(arg_ptrs); // Drop so that usage is not allowed after unmapping context

                let contexts = context::contexts();
                let (vfork, ppid) = {
                    let context_lock = contexts.current().ok_or(Error::new(ESRCH))?;
                    let mut context = context_lock.write();

                    // Unmap previous image and stack
                    context.image.clear();
                    drop(context.heap.take());
                    drop(context.stack.take());
                    context.grants = Arc::new(Mutex::new(Vec::new()));

                    if stat.st_mode & syscall::flag::MODE_SETUID == syscall::flag::MODE_SETUID {
                        context.uid = stat.st_uid;
                    }

                    if stat.st_mode & syscall::flag::MODE_SETGID == syscall::flag::MODE_SETGID {
                        context.gid = stat.st_gid;
                    }

                    // Map and copy new segments
                    for segment in elf.segments() {
                        if segment.p_type == program_header::PT_LOAD {
                            let mut memory = context::memory::Memory::new(
                                VirtualAddress::new(segment.p_vaddr as usize),
                                segment.p_memsz as usize,
                                entry::NO_EXECUTE | entry::WRITABLE,
                                true,
                                true
                            );

                            unsafe {
                                // Copy file data
                                memcpy(segment.p_vaddr as *mut u8,
                                        (elf.data.as_ptr() as usize + segment.p_offset as usize) as *const u8,
                                        segment.p_filesz as usize);
                            }

                            let mut flags = entry::NO_EXECUTE | entry::USER_ACCESSIBLE;

                            if segment.p_flags & program_header::PF_R == program_header::PF_R {
                                flags.insert(entry::PRESENT);
                            }

                            // W ^ X. If it is executable, do not allow it to be writable, even if requested
                            if segment.p_flags & program_header::PF_X == program_header::PF_X {
                                flags.remove(entry::NO_EXECUTE);
                            } else if segment.p_flags & program_header::PF_W == program_header::PF_W {
                                flags.insert(entry::WRITABLE);
                            }

                            memory.remap(flags, true);

                            context.image.push(memory.to_shared());
                        }
                    }

                    // Map heap
                    context.heap = Some(context::memory::Memory::new(
                        VirtualAddress::new(arch::USER_HEAP_OFFSET),
                        0,
                        entry::NO_EXECUTE | entry::WRITABLE | entry::USER_ACCESSIBLE,
                        true,
                        true
                    ).to_shared());

                    // Map stack
                    context.stack = Some(context::memory::Memory::new(
                        VirtualAddress::new(arch::USER_STACK_OFFSET),
                        arch::USER_STACK_SIZE,
                        entry::NO_EXECUTE | entry::WRITABLE | entry::USER_ACCESSIBLE,
                        true,
                        true
                    ));

                    let mut arg_size = 0;
                    for arg in args.iter().rev() {
                        sp -= mem::size_of::<usize>();
                        unsafe { *(sp as *mut usize) = arch::USER_ARG_OFFSET + arg_size; }
                        sp -= mem::size_of::<usize>();
                        unsafe { *(sp as *mut usize) = arg.len(); }

                        arg_size += arg.len();
                    }

                    sp -= mem::size_of::<usize>();
                    unsafe { *(sp as *mut usize) = args.len(); }

                    if arg_size > 0 {
                        let mut memory = context::memory::Memory::new(
                            VirtualAddress::new(arch::USER_ARG_OFFSET),
                            arg_size,
                            entry::NO_EXECUTE | entry::WRITABLE,
                            true,
                            true
                        );

                        let mut arg_offset = 0;
                        for arg in args.iter().rev() {
                            unsafe {
                                memcpy((arch::USER_ARG_OFFSET + arg_offset) as *mut u8,
                                       arg.as_ptr(),
                                       arg.len());
                            }

                            arg_offset += arg.len();
                        }

                        memory.remap(entry::NO_EXECUTE | entry::USER_ACCESSIBLE, true);

                        context.image.push(memory.to_shared());
                    }

                    let vfork = context.vfork;
                    context.vfork = false;
                    (vfork, context.ppid)
                };

                if vfork {
                    if let Some(context_lock) = contexts.get(ppid) {
                        let mut context = context_lock.write();
                        if context.status == context::Status::Blocked {
                            context.status = context::Status::Runnable;
                        } else {
                            println!("{} not blocked for exec vfork unblock", ppid);
                        }
                    } else {
                        println!("{} not found for exec vfork unblock", ppid);
                    }
                }
            },
            Err(err) => {
                println!("failed to execute {}: {}", unsafe { str::from_utf8_unchecked(path) }, err);
                return Err(Error::new(ENOEXEC));
            }
        }
    }

    // Go to usermode
    unsafe { usermode(entry, sp); }
}

pub fn getgid() -> Result<usize> {
    let contexts = context::contexts();
    let context_lock = contexts.current().ok_or(Error::new(ESRCH))?;
    let context = context_lock.read();
    Ok(context.gid as usize)
}

pub fn getpid() -> Result<usize> {
    let contexts = context::contexts();
    let context_lock = contexts.current().ok_or(Error::new(ESRCH))?;
    let context = context_lock.read();
    Ok(context.id)
}

pub fn getuid() -> Result<usize> {
    let contexts = context::contexts();
    let context_lock = contexts.current().ok_or(Error::new(ESRCH))?;
    let context = context_lock.read();
    Ok(context.uid as usize)
}

pub fn iopl(_level: usize) -> Result<usize> {
    //TODO
    Ok(0)
}

pub fn physalloc(size: usize) -> Result<usize> {
    allocate_frames((size + 4095)/4096).ok_or(Error::new(ENOMEM)).map(|frame| frame.start_address().get())
}

pub fn physfree(physical_address: usize, size: usize) -> Result<usize> {
    deallocate_frames(Frame::containing_address(PhysicalAddress::new(physical_address)), (size + 4095)/4096);
    //TODO: Check that no double free occured
    Ok(0)
}

//TODO: verify exlusive access to physical memory
pub fn physmap(physical_address: usize, size: usize, flags: usize) -> Result<usize> {
    if size == 0 {
        Ok(0)
    } else {
        let contexts = context::contexts();
        let context_lock = contexts.current().ok_or(Error::new(ESRCH))?;
        let context = context_lock.read();

        let mut grants = context.grants.lock();

        let from_address = (physical_address/4096) * 4096;
        let offset = physical_address - from_address;
        let full_size = ((offset + size + 4095)/4096) * 4096;
        let mut to_address = arch::USER_GRANT_OFFSET;

        let mut entry_flags = entry::PRESENT | entry::NO_EXECUTE | entry::USER_ACCESSIBLE;
        if flags & MAP_WRITE == MAP_WRITE {
            entry_flags |= entry::WRITABLE;
        }
        if flags & MAP_WRITE_COMBINE == MAP_WRITE_COMBINE {
            entry_flags |= entry::HUGE_PAGE;
        }

        for i in 0 .. grants.len() {
            let start = grants[i].start_address().get();
            if to_address + full_size < start {
                grants.insert(i, Grant::physmap(
                    PhysicalAddress::new(from_address),
                    VirtualAddress::new(to_address),
                    full_size,
                    entry_flags
                ));

                return Ok(to_address + offset);
            } else {
                let pages = (grants[i].size() + 4095) / 4096;
                let end = start + pages * 4096;
                to_address = end;
            }
        }

        grants.push(Grant::physmap(
            PhysicalAddress::new(from_address),
            VirtualAddress::new(to_address),
            full_size,
            entry_flags
        ));

        Ok(to_address + offset)
    }
}

pub fn physunmap(virtual_address: usize) -> Result<usize> {
    if virtual_address == 0 {
        Ok(0)
    } else {
        let contexts = context::contexts();
        let context_lock = contexts.current().ok_or(Error::new(ESRCH))?;
        let context = context_lock.read();

        let mut grants = context.grants.lock();

        for i in 0 .. grants.len() {
            let start = grants[i].start_address().get();
            let end = start + grants[i].size();
            if virtual_address >= start && virtual_address < end {
                grants.remove(i).unmap();

                return Ok(0);
            }
        }

        Err(Error::new(EFAULT))
    }
}

pub fn sched_yield() -> Result<usize> {
    unsafe { context::switch(); }
    Ok(0)
}

pub fn setgid(gid: u32) -> Result<usize> {
    let contexts = context::contexts();
    let context_lock = contexts.current().ok_or(Error::new(ESRCH))?;
    let mut context = context_lock.write();
    if context.gid == 0 {
        context.gid = gid;
        Ok(0)
    } else if context.gid == gid {
        Ok(0)
    } else {
        Err(Error::new(EPERM))
    }
}

pub fn setuid(uid: u32) -> Result<usize> {
    let contexts = context::contexts();
    let context_lock = contexts.current().ok_or(Error::new(ESRCH))?;
    let mut context = context_lock.write();
    if context.uid == 0 {
        context.uid = uid;
        Ok(0)
    } else if context.uid == uid {
        Ok(0)
    } else {
        Err(Error::new(EPERM))
    }
}

pub fn virttophys(virtual_address: usize) -> Result<usize> {
    let active_table = unsafe { ActivePageTable::new() };
    match active_table.translate(VirtualAddress::new(virtual_address)) {
        Some(physical_address) => Ok(physical_address.get()),
        None => Err(Error::new(EFAULT))
    }
}

pub fn waitpid(pid: usize, status_ptr: usize, flags: usize) -> Result<usize> {
    loop {
        {
            let mut exited = false;

            {
                let contexts = context::contexts();
                let context_lock = contexts.get(pid).ok_or(Error::new(ESRCH))?;
                let context = context_lock.read();
                if let context::Status::Exited(status) = context.status {
                    if status_ptr != 0 {
                        let status_slice = validate_slice_mut(status_ptr as *mut usize, 1)?;
                        status_slice[0] = status;
                    }
                    exited = true;
                }
            }

            if exited {
                let mut contexts = context::contexts_mut();
                return contexts.remove(pid).ok_or(Error::new(ESRCH)).and(Ok(pid));
            } else if flags & WNOHANG == WNOHANG {
                return Ok(0);
            }
        }

        unsafe { context::switch(); } //TODO: Block
    }
}
