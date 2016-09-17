///! Process syscalls

use alloc::arc::Arc;
use core::mem;
use core::str;
use spin::Mutex;

use arch;
use arch::memory::allocate_frame;
use arch::paging::{ActivePageTable, InactivePageTable, Page, VirtualAddress, entry};
use arch::paging::temporary_page::TemporaryPage;
use context;
use elf;
use scheme;
use syscall::{self, Error, Result};

pub fn brk(address: usize) -> Result<usize> {
    let contexts = context::contexts();
    let context_lock = contexts.current().ok_or(Error::NoProcess)?;
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
        //TODO: Return correct error
        Err(Error::NotPermitted)
    }
}

pub const CLONE_VM: usize = 0x100;
pub const CLONE_FS: usize = 0x200;
pub const CLONE_FILES: usize = 0x400;
pub const CLONE_VFORK: usize = 0x4000;
pub fn clone(flags: usize, stack_base: usize) -> Result<usize> {
    //TODO: Implement flags
    //TODO: Copy on write?
    println!("Clone {:X}: {:X}", flags, stack_base);

    let pid;
    {
        let arch;
        let mut kstack_option = None;
        let mut offset = 0;
        let mut image = vec![];
        let mut heap_option = None;
        let mut stack_option = None;
        let mut files = Arc::new(Mutex::new(vec![]));

        // Copy from old process
        {
            let contexts = context::contexts();
            let context_lock = contexts.current().ok_or(Error::NoProcess)?;
            let context = context_lock.read();
            arch = context.arch.clone();

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
                    image.push(memory_shared.borrow());
                }

                if let Some(ref heap_shared) = context.heap {
                    heap_option = Some(heap_shared.borrow());
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

            if flags & CLONE_FILES == CLONE_FILES {
                files = context.files.clone();
            } else {
                for (fd, file_option) in context.files.lock().iter().enumerate() {
                    if let Some(file) = *file_option {
                        let result = {
                            let schemes = scheme::schemes();
                            let scheme_mutex = schemes.get(file.scheme).ok_or(Error::BadFile)?;
                            let result = scheme_mutex.lock().dup(file.number);
                            result
                        };
                        match result {
                            Ok(new_number) => {
                                files.lock().push(Some(context::file::File { scheme: file.scheme, number: new_number }));
                            },
                            Err(err) => {
                                println!("clone: failed to dup {}: {:?}", fd, err);
                            }
                        }
                    } else {
                        files.lock().push(None);
                    }
                }
            }
        }

        // Set up new process
        {
            let mut contexts = context::contexts_mut();
            let context_lock = contexts.new_context()?;
            let mut context = context_lock.write();

            pid = context.id;

            context.arch = arch;

            let mut active_table = unsafe { ActivePageTable::new() };

            let mut temporary_page = TemporaryPage::new(Page::containing_address(VirtualAddress::new(0x8_0000_0000)));

            let mut new_table = {
                let frame = allocate_frame().expect("no more frames in syscall::clone new_table");
                InactivePageTable::new(frame, &mut active_table, &mut temporary_page)
            };

            // Copy kernel mapping
            {
                let frame = active_table.p4()[510].pointed_frame().expect("kernel table not mapped");
                let flags = active_table.p4()[510].flags();
                active_table.with(&mut new_table, &mut temporary_page, |mapper| {
                    mapper.p4_mut()[510].set(frame, flags);
                });
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
                    let flags = active_table.p4()[0].flags();
                    active_table.with(&mut new_table, &mut temporary_page, |mapper| {
                        mapper.p4_mut()[1].set(frame, flags);
                    });
                    context.heap = Some(heap_shared);
                }
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

            context.files = files;

            context.arch.set_page_table(unsafe { new_table.address() });

            context.status = context::Status::Runnable;
        }
    }

    unsafe { context::switch(); }

    Ok(pid)
}

pub fn exit(status: usize) -> ! {
    println!("Exit {}", status);

    {
        let contexts = context::contexts();
        let context_lock = contexts.current().expect("tried to exit without context");
        let mut context = context_lock.write();
        context.image.clear();
        drop(context.heap.take());
        drop(context.stack.take());
        context.status = context::Status::Exited;
    }

    unsafe { context::switch(); }

    unreachable!();
}

pub fn exec(path: &[u8], _args: &[[usize; 2]]) -> Result<usize> {
    //TODO: Use args
    //TODO: Unmap previous mappings
    //TODO: Drop data vec
    println!("Exec {}", unsafe { str::from_utf8_unchecked(path) });

    let file = syscall::open(path, 0)?;
    let mut data = vec![];
    loop {
        let mut buf = [0; 4096];
        let count = syscall::read(file, &mut buf)?;
        if count > 0 {
            data.extend_from_slice(&buf[..count]);
        } else {
            break;
        }
    }
    let _ = syscall::close(file);

    match elf::Elf::from(&data) {
        Ok(elf) => {
            elf.run().and(Ok(0))
        },
        Err(err) => {
            println!("failed to execute {}: {}", unsafe { str::from_utf8_unchecked(path) }, err);
            Err(Error::NoExec)
        }
    }
}

pub fn getpid() -> Result<usize> {
    let contexts = context::contexts();
    let context_lock = contexts.current().ok_or(Error::NoProcess)?;
    let context = context_lock.read();
    Ok(context.id)
}

pub fn iopl(_level: usize) -> Result<usize> {
    //TODO
    Ok(0)
}

pub fn sched_yield() -> Result<usize> {
    unsafe { context::switch(); }
    Ok(0)
}

pub fn waitpid(pid: usize, _status_ptr: usize, _options: usize) -> Result<usize> {
    loop {
        {
            let mut exited = false;

            {
                let contexts = context::contexts();
                let context_lock = contexts.get(pid).ok_or(Error::NoProcess)?;
                let context = context_lock.read();
                if context.status == context::Status::Exited {
                    exited = true;
                }
            }

            if exited {
                let mut contexts = context::contexts_mut();
                return contexts.remove(pid).ok_or(Error::NoProcess).and(Ok(pid));
            }
        }

        unsafe { context::switch(); }
    }
}
