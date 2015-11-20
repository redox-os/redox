use collections::string::ToString;
use collections::vec::Vec;

use core::ops::Deref;
use core::{ptr, slice, str, usize};

use common::get_slice::GetSlice;
use common::memory;
use common::time::Duration;

use drivers::pio::*;

use programs::executor::execute;

use scheduler::{self, Regs};
use scheduler::context::{context_clone, context_exit, context_switch, Context, ContextMemory,
                         ContextFile};

use schemes::{Resource, ResourceSeek, Url};

use syscall::common::*;

/// Helper function for handling C strings, please do not copy it or make it pub or change it
unsafe fn c_string_to_slice<'a>(ptr: *const u8) -> &'a [u8] {
    if ptr > 0 as *const u8 {
        let mut len = 0;
        while ptr::read(ptr.offset(len as isize)) > 0 {
            len += 1;
        }

        slice::from_raw_parts(ptr, len)
    } else {
        &[]
    }
}
/// Helper function for handling C strings, please do not copy it or make it pub or change it
unsafe fn c_array_to_slice<'a>(ptr: *const *const u8) -> &'a [*const u8] {
    if ptr > 0 as *const *const u8 {
        let mut len = 0;
        while ptr::read(ptr.offset(len as isize)) > 0 as *const u8 {
            len += 1;
        }

        slice::from_raw_parts(ptr, len)
    } else {
        &[]
    }
}

pub unsafe fn do_sys_debug(ptr: *const u8, len: usize) {
    let bytes = slice::from_raw_parts(ptr, len);

    let reenable = scheduler::start_no_ints();

    if ::console as usize > 0 {
        (*::console).write(bytes);
    }

    let serial_status = Pio8::new(0x3F8 + 5);
    let mut serial_data = Pio8::new(0x3F8);

    for byte in bytes.iter() {
        while serial_status.read() & 0x20 == 0 {}
        serial_data.write(*byte);

        if *byte == 8 {
            while serial_status.read() & 0x20 == 0 {}
            serial_data.write(0x20);

            while serial_status.read() & 0x20 == 0 {}
            serial_data.write(8);
        }
    }

    scheduler::end_no_ints(reenable);
}

pub unsafe fn do_sys_brk(addr: usize) -> usize {
    let mut ret = 0;

    let reenable = scheduler::start_no_ints();

    if let Some(mut current) = Context::current_mut() {
        current.unmap();

        ret = current.next_mem();

        // TODO: Make this smarter, currently it attempt to resize the entire data segment
        if let Some(mut mem) = (*current.memory.get()).last_mut() {
            if mem.writeable {
                if addr >= mem.virtual_address {
                    let size = addr - mem.virtual_address;
                    let physical_address = memory::realloc(mem.physical_address, size);
                    if physical_address > 0 {
                        mem.physical_address = physical_address;
                        mem.virtual_size = size;
                        ret = mem.virtual_address + mem.virtual_size;
                    } else {
                        mem.virtual_size = 0;
                        debug!("BRK: Realloc failed {:X}, {}\n", mem.virtual_address, size);
                    }
                }
            } else {
                debug!("BRK: End segment not writeable\n");
            }
        }

        current.clean_mem();
        current.map();
    } else {
        debug!("BRK: Context not found\n");
    }

    scheduler::end_no_ints(reenable);

    ret
}

pub unsafe extern "cdecl" fn do_sys_chdir(path: *const u8) -> usize {
    let mut ret = usize::MAX;

    let reenable = scheduler::start_no_ints();

    if let Some(current) = Context::current() {
        *current.cwd.get() =
            current.canonicalize(&str::from_utf8_unchecked(&c_string_to_slice(path)));
        ret = 0;
    }

    scheduler::end_no_ints(reenable);

    ret
}

#[cold]
#[inline(never)]
pub unsafe fn do_sys_clone(flags: usize) -> usize {
    let mut parent_ptr: *const Context = 0 as *const Context;

    let reenable = scheduler::start_no_ints();

    if let Some(parent) = Context::current() {
        parent_ptr = parent.deref();

        let mut context_clone_args: Vec<usize> = Vec::new();
        context_clone_args.push(flags);
        context_clone_args.push(parent_ptr as usize);
        context_clone_args.push(context_exit as usize);

        let contexts = &mut *::scheduler::context::contexts_ptr;
        contexts.push(Context::new(format!("kclone {}", parent.name),
                                   false,
                                   context_clone as usize,
                                   &context_clone_args));
    }

    scheduler::end_no_ints(reenable);

    context_switch(false);

    let mut ret = usize::MAX;

    if parent_ptr as usize > 0 {
        let reenable = scheduler::start_no_ints();

        if let Some(new) = Context::current() {
            let new_ptr: *const Context = new.deref();
            if new_ptr == parent_ptr {
                ret = 1;
            } else {
                ret = 0;
            }
        }

        scheduler::end_no_ints(reenable);
    }

    ret
}

pub unsafe fn do_sys_close(fd: usize) -> usize {
    let mut ret = usize::MAX;

    let reenable = scheduler::start_no_ints();

    if let Some(current) = Context::current() {
        for i in 0..(*current.files.get()).len() {
            let mut remove = false;
            if let Some(file) = (*current.files.get()).get(i) {
                if file.fd == fd {
                    remove = true;
                }
            }

            if remove {
                if i < (*current.files.get()).len() {
                    let file = (*current.files.get()).remove(i);

                    scheduler::end_no_ints(reenable);

                    drop(file);

                    scheduler::start_no_ints();

                    ret = 0;
                }

                break;
            }
        }
    }

    scheduler::end_no_ints(reenable);

    ret
}

pub unsafe fn do_sys_clock_gettime(clock: usize, tp: *mut TimeSpec) -> usize {
    let mut ret = usize::MAX;

    let reenable = scheduler::start_no_ints();

    if tp as usize > 0 {
        match clock {
            CLOCK_REALTIME => {
                (*tp).tv_sec = ::clock_realtime.secs;
                (*tp).tv_nsec = ::clock_realtime.nanos;
                ret = 0;
            }
            CLOCK_MONOTONIC => {
                (*tp).tv_sec = ::clock_monotonic.secs;
                (*tp).tv_nsec = ::clock_monotonic.nanos;
                ret = 0;
            }
            _ => (),
        }
    }

    scheduler::end_no_ints(reenable);

    ret
}

pub unsafe fn do_sys_dup(fd: usize) -> usize {
    let mut ret = usize::MAX;

    let reenable = scheduler::start_no_ints();

    if let Some(current) = Context::current() {
        let new_fd = current.next_fd();

        if let Some(resource) = current.get_file(fd) {
            if let Some(new_resource) = resource.dup() {
                ret = new_fd;
                (*current.files.get()).push(ContextFile {
                    fd: new_fd,
                    resource: new_resource,
                });
            }
        }
    }

    scheduler::end_no_ints(reenable);

    ret
}

// TODO: Make sure this does not return (it should be called from a clone)
pub unsafe fn do_sys_execve(path: *const u8, args: *const *const u8) -> usize {
    let mut ret = usize::MAX;

    let reenable = scheduler::start_no_ints();

    if let Some(current) = Context::current() {
        let path_string = current.canonicalize(str::from_utf8_unchecked(c_string_to_slice(path)));

        let path = Url::from_string(path_string.clone());
        let wd = Url::from_string(path_string.get_slice(None,
                                                        Some(path_string.rfind('/').unwrap_or(0) +
                                                             1))
                                             .to_string());

        let mut args_vec = Vec::new();
        for arg in c_array_to_slice(args) {
            args_vec.push(str::from_utf8_unchecked(c_string_to_slice(*arg)).to_string());
        }

        execute(&path, &wd, args_vec);
        ret = 0;
    }

    scheduler::end_no_ints(reenable);

    ret
}

pub unsafe fn do_sys_fpath(fd: usize, buf: *mut u8, len: usize) -> usize {
    let mut ret = usize::MAX;

    let reenable = scheduler::start_no_ints();

    if let Some(current) = Context::current() {
        if let Some(resource) = current.get_file(fd) {
            scheduler::end_no_ints(reenable);

            ret = 0;
            // TODO: Improve performance
            for b in resource.url().to_string().as_bytes().iter() {
                if ret < len {
                    ptr::write(buf.offset(ret as isize), *b);
                } else {
                    break;
                }
                ret += 1;
            }

            scheduler::start_no_ints();
        }
    }

    scheduler::end_no_ints(reenable);

    ret
}

pub unsafe fn do_sys_fsync(fd: usize) -> usize {
    let mut ret = usize::MAX;

    let reenable = scheduler::start_no_ints();

    if let Some(mut current) = Context::current_mut() {
        if let Some(mut resource) = current.get_file_mut(fd) {
            scheduler::end_no_ints(reenable);

            if resource.sync() {
                ret = 0;
            }

            scheduler::start_no_ints();
        }
    }

    scheduler::end_no_ints(reenable);

    ret
}

pub unsafe fn do_sys_ftruncate(fd: usize, len: usize) -> usize {
    let mut ret = usize::MAX;

    let reenable = scheduler::start_no_ints();

    if let Some(mut current) = Context::current_mut() {
        if let Some(mut resource) = current.get_file_mut(fd) {
            scheduler::end_no_ints(reenable);

            if resource.truncate(len) {
                ret = 0;
            }

            scheduler::start_no_ints();
        }
    }

    scheduler::end_no_ints(reenable);

    ret
}

// TODO: link

pub unsafe fn do_sys_lseek(fd: usize, offset: isize, whence: usize) -> usize {
    let mut ret = usize::MAX;

    let reenable = scheduler::start_no_ints();

    if let Some(mut current) = Context::current_mut() {
        if let Some(mut resource) = current.get_file_mut(fd) {
            scheduler::end_no_ints(reenable);

            match whence {
                SEEK_SET =>
                    if let Some(count) = resource.seek(ResourceSeek::Start(offset as usize)) {
                        ret = count;
                    },
                SEEK_CUR => if let Some(count) = resource.seek(ResourceSeek::Current(offset)) {
                    ret = count;
                },
                SEEK_END => if let Some(count) = resource.seek(ResourceSeek::End(offset)) {
                    ret = count;
                },
                _ => (),
            }

            scheduler::start_no_ints();
        }
    }

    scheduler::end_no_ints(reenable);

    ret
}

pub unsafe fn do_sys_mkdir(path: *const u8, mode: usize) -> usize {
    let mut ret = usize::MAX;

    // Implement body of do_sys_mkdir

    ret
}

pub unsafe fn do_sys_nanosleep(req: *const TimeSpec, rem: *mut TimeSpec) -> usize {
    if req as usize > 0 {
        Duration::new((*req).tv_sec, (*req).tv_nsec).sleep();

        if rem as usize > 0 {
            (*rem).tv_sec = 0;
            (*rem).tv_nsec = 0;
        }

        0
    } else {
        usize::MAX
    }
}

pub unsafe fn do_sys_open(path: *const u8, flags: usize) -> usize {
    let mut fd = usize::MAX;

    let reenable = scheduler::start_no_ints();

    if let Some(current) = Context::current() {
        let path_string = current.canonicalize(str::from_utf8_unchecked(c_string_to_slice(path)));

        scheduler::end_no_ints(reenable);

        let resource_option = (*::session_ptr).open(&Url::from_string(path_string), flags);

        scheduler::start_no_ints();

        if let Some(resource) = resource_option {
            fd = current.next_fd();

            (*current.files.get()).push(ContextFile {
                fd: fd,
                resource: resource,
            });
        }
    }

    scheduler::end_no_ints(reenable);

    fd
}

pub unsafe fn do_sys_read(fd: usize, buf: *mut u8, count: usize) -> usize {
    let mut ret = usize::MAX;

    let reenable = scheduler::start_no_ints();

    if let Some(mut current) = Context::current_mut() {
        if let Some(resource) = current.get_file_mut(fd) {
            scheduler::end_no_ints(reenable);

            if let Some(count) = resource.read(slice::from_raw_parts_mut(buf, count)) {
                ret = count;
            }

            scheduler::start_no_ints();
        }
    }

    scheduler::end_no_ints(reenable);

    ret
}

pub unsafe fn do_sys_unlink(path: *const u8) -> usize {
    let mut ret = usize::MAX;

    // Implement body of do_sys_mkdir

    ret
}

pub unsafe fn do_sys_write(fd: usize, buf: *const u8, count: usize) -> usize {
    let mut ret = usize::MAX;

    let reenable = scheduler::start_no_ints();

    if let Some(mut current) = Context::current_mut() {
        if let Some(resource) = current.get_file_mut(fd) {
            scheduler::end_no_ints(reenable);

            if let Some(count) = resource.write(slice::from_raw_parts(buf, count)) {
                ret = count;
            }

            scheduler::start_no_ints();
        }
    }

    scheduler::end_no_ints(reenable);

    ret
}

pub unsafe fn do_sys_alloc(size: usize) -> usize {
    let mut ret = 0;

    let reenable = scheduler::start_no_ints();

    if let Some(mut current) = Context::current_mut() {
        current.unmap();
        let physical_address = memory::alloc(size);
        if physical_address > 0 {
            ret = current.next_mem();
            (*current.memory.get()).push(ContextMemory {
                physical_address: physical_address,
                virtual_address: ret,
                virtual_size: size,
                writeable: true,
            });
        }
        current.clean_mem();
        current.map();
    }

    scheduler::end_no_ints(reenable);

    ret
}

pub unsafe fn do_sys_realloc(ptr: usize, size: usize) -> usize {
    let mut ret = 0;

    let reenable = scheduler::start_no_ints();

    if let Some(mut current) = Context::current_mut() {
        current.unmap();
        if let Some(mut mem) = current.get_mem_mut(ptr) {
            let physical_address = memory::realloc(mem.physical_address, size);
            if physical_address > 0 {
                mem.physical_address = physical_address;
                mem.virtual_size = size;
                ret = mem.virtual_address;
            } else {
                mem.virtual_size = 0;
            }
        }
        current.clean_mem();
        current.map();
    }

    scheduler::end_no_ints(reenable);

    ret
}

pub unsafe fn do_sys_realloc_inplace(ptr: usize, size: usize) -> usize {
    let mut ret = 0;

    let reenable = scheduler::start_no_ints();

    if let Some(mut current) = Context::current_mut() {
        current.unmap();
        if let Some(mut mem) = current.get_mem_mut(ptr) {
            mem.virtual_size = memory::realloc_inplace(mem.physical_address, size);
            ret = mem.virtual_size;
        }
        current.clean_mem();
        current.map();
    }

    scheduler::end_no_ints(reenable);

    ret
}

pub unsafe fn do_sys_unalloc(ptr: usize) {
    let reenable = scheduler::start_no_ints();

    if let Some(mut current) = Context::current_mut() {
        current.unmap();
        if let Some(mut mem) = current.get_mem_mut(ptr) {
            mem.virtual_size = 0;
        }
        current.clean_mem();
        current.map();
    }

    scheduler::end_no_ints(reenable);
}

pub unsafe fn syscall_handle(regs: &mut Regs) -> bool {
    match regs.ax {
        SYS_DEBUG => do_sys_debug(regs.bx as *const u8, regs.cx),
        // Linux
        SYS_BRK => regs.ax = do_sys_brk(regs.bx),
        SYS_CHDIR => regs.ax = do_sys_chdir(regs.bx as *const u8),
        SYS_CLONE => regs.ax = do_sys_clone(regs.bx),
        SYS_CLOSE => regs.ax = do_sys_close(regs.bx as usize),
        SYS_CLOCK_GETTIME => regs.ax = do_sys_clock_gettime(regs.bx, regs.cx as *mut TimeSpec),
        SYS_DUP => regs.ax = do_sys_dup(regs.bx),
        SYS_EXECVE => regs.ax = do_sys_execve(regs.bx as *const u8, regs.cx as *const *const u8),
        SYS_EXIT => context_exit(),
        SYS_FPATH => regs.ax = do_sys_fpath(regs.bx, regs.cx as *mut u8, regs.dx),
        // TODO: fstat
        SYS_FSYNC => regs.ax = do_sys_fsync(regs.bx),
        SYS_FTRUNCATE => regs.ax = do_sys_ftruncate(regs.bx, regs.cx),
        // TODO: link
        SYS_LSEEK => regs.ax = do_sys_lseek(regs.bx, regs.cx as isize, regs.dx as usize),
        SYS_MKDIR => regs.ax = do_sys_mkdir(regs.bx as *const u8, regs.cx),
        SYS_NANOSLEEP =>
            regs.ax = do_sys_nanosleep(regs.bx as *const TimeSpec, regs.cx as *mut TimeSpec),
        SYS_OPEN => regs.ax = do_sys_open(regs.bx as *const u8, regs.cx), //regs.cx as isize, regs.dx as isize),
        SYS_READ => regs.ax = do_sys_read(regs.bx, regs.cx as *mut u8, regs.dx),
        SYS_UNLINK => regs.ax = do_sys_unlink(regs.bx as *const u8),
        SYS_WRITE => regs.ax = do_sys_write(regs.bx, regs.cx as *mut u8, regs.dx),
        SYS_YIELD => context_switch(false),

        // Rust Memory
        SYS_ALLOC => regs.ax = do_sys_alloc(regs.bx),
        SYS_REALLOC => regs.ax = do_sys_realloc(regs.bx, regs.cx),
        SYS_REALLOC_INPLACE => regs.ax = do_sys_realloc_inplace(regs.bx, regs.cx),
        SYS_UNALLOC => do_sys_unalloc(regs.bx),

        _ => return false,
    }

    true
}
