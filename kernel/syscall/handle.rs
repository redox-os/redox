use alloc::arc::Arc;

use collections::string::{String, ToString};
use collections::vec::Vec;

use core::ops::Deref;
use core::{mem, ptr, slice, str, usize};

use common::memory;
use common::time::Duration;

use drivers::pio::*;

use programs::executor::execute;

use scheduler::{self, Regs};
use scheduler::context::{context_clone, context_i, context_switch, Context, ContextMemory, ContextFile,
                         ContextStatus};

use schemes::{Resource, ResourceSeek, Url};

use sync::Intex;

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

    if ::ENV_PTR.is_some() {
        ::env().console.lock().write(bytes);
    } else {
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
    }
}

pub unsafe fn do_sys_brk(addr: usize) -> usize {
    let mut ret = 0;

    let mut contexts = ::env().contexts.lock();
    if let Some(mut current) = contexts.get_mut(context_i) {
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

    ret
}

pub unsafe extern "cdecl" fn do_sys_chdir(path: *const u8) -> usize {
    let mut contexts = ::env().contexts.lock();
    if let Some(mut current) = contexts.get_mut(context_i) {
        *current.cwd.get() =
            current.canonicalize(&str::from_utf8_unchecked(&c_string_to_slice(path)));
        return 0;
    }

    usize::MAX
}

#[cold]
#[inline(never)]
pub unsafe fn do_sys_clone(flags: usize) -> usize {
    let mut clone_pid = usize::MAX;
    let mut mem_count = 0;

    {
        let mut contexts = ::env().contexts.lock();

        let child_option = if let Some(parent) = contexts.get(context_i) {
            clone_pid = Context::next_pid();
            mem_count = Arc::strong_count(&parent.memory);

            let parent_ptr: *const Context = parent.deref();

            let mut context_clone_args: Vec<usize> = Vec::new();
            context_clone_args.push(clone_pid);
            context_clone_args.push(flags);
            context_clone_args.push(parent_ptr as usize);
            context_clone_args.push(0); //Return address, 0 catches bad code

            Some(Context::new(format!("kclone {}", parent.name),
                                       context_clone as usize,
                                       &context_clone_args))
        } else {
            None
        };

        if let Some(child) = child_option {
            contexts.push(child);
        }
    }

    context_switch(false);

    let mut ret = usize::MAX;

    if clone_pid != usize::MAX {
        let contexts = ::env().contexts.lock();
        if let Some(current) = contexts.get(context_i) {
            if current.pid == clone_pid {
                ret = 0;
            } else {
                if flags & CLONE_VFORK == CLONE_VFORK {
                    while Arc::strong_count(&current.memory) > mem_count {
                        context_switch(false);
                    }
                }
                ret = clone_pid;
            }
        }
    }

    ret
}

pub unsafe fn do_sys_close(fd: usize) -> usize {
    let mut contexts = ::env().contexts.lock();
    if let Some(mut current) = contexts.get_mut(context_i) {
        for i in 0..(*current.files.get()).len() {
            let mut remove = false;
            if let Some(file) = (*current.files.get()).get(i) {
                if file.fd == fd {
                    remove = true;
                }
            }

            if remove {
                if i < (*current.files.get()).len() {
                    drop((*current.files.get()).remove(i));

                    return 0;
                }

                break;
            }
        }
    }

    usize::MAX
}

pub unsafe fn do_sys_clock_gettime(clock: usize, tp: *mut TimeSpec) -> usize {
    let _intex = Intex::static_lock();

    if tp as usize > 0 {
        let env = ::env();
        match clock {
            CLOCK_REALTIME => {
                (*tp).tv_sec = env.clock_realtime.secs;
                (*tp).tv_nsec = env.clock_realtime.nanos;
                return 0;
            }
            CLOCK_MONOTONIC => {
                (*tp).tv_sec = env.clock_monotonic.secs;
                (*tp).tv_nsec = env.clock_monotonic.nanos;
                return 0;
            }
            _ => (),
        }
    }

    usize::MAX
}

pub unsafe fn do_sys_dup(fd: usize) -> usize {
    let mut contexts = ::env().contexts.lock();
    if let Some(mut current) = contexts.get_mut(context_i) {
        if let Some(resource) = current.get_file(fd) {
            if let Some(new_resource) = resource.dup() {
                let new_fd = current.next_fd();

                (*current.files.get()).push(ContextFile {
                    fd: new_fd,
                    resource: new_resource,
                });

                return new_fd;
            }
        }
    }

    usize::MAX
}

pub unsafe fn do_sys_execve(path: *const u8, args: *const *const u8) -> usize {
    let mut args_vec = Vec::new();
    let path_url = {
        let contexts = ::env().contexts.lock();
        if let Some(current) = contexts.get(context_i) {
            for arg in c_array_to_slice(args) {
                args_vec.push(str::from_utf8_unchecked(c_string_to_slice(*arg)).to_string());
            }

            Url::from_string(current.canonicalize(str::from_utf8_unchecked(c_string_to_slice(path))))
        } else {
            Url::from_string(String::new())
        }
    };

    execute(path_url, args_vec);

    usize::MAX
}

pub unsafe fn do_sys_spawnve(path: *const u8, args: *const *const u8) -> usize {
    let mut args_vec = Vec::new();
    let path_url = {
        let contexts = ::env().contexts.lock();
        if let Some(current) = contexts.get(context_i) {
            for arg in c_array_to_slice(args) {
                args_vec.push(str::from_utf8_unchecked(c_string_to_slice(*arg)).to_string());
            }

            Url::from_string(current.canonicalize(str::from_utf8_unchecked(c_string_to_slice(path))))
        } else {
            Url::from_string(String::new())
        }
    };

    return Context::spawn("kspawn".to_string(),
                          box move || {
                              let wd_c = "file:/\0";
                              do_sys_chdir(wd_c.as_ptr());

                              let stdio_c = "debug:\0";
                              do_sys_open(stdio_c.as_ptr(), 0);
                              do_sys_open(stdio_c.as_ptr(), 0);
                              do_sys_open(stdio_c.as_ptr(), 0);

                              execute(path_url, args_vec);

                              do_sys_exit(127);
                          });
}

/// Exit context
///
/// Unsafe due to interrupt disabling and raw pointers
pub unsafe fn do_sys_exit(status: usize) {
    {
        let mut contexts = ::env().contexts.lock();

        let mut statuses = Vec::new();
        let (pid, ppid) = {
            if let Some(mut current) = contexts.get_mut(context_i) {
                current.exited = true;
                mem::swap(&mut statuses, &mut current.statuses);
                (current.pid, current.ppid)
            } else {
                (0, 0)
            }
        };

        for mut context in contexts.iter_mut() {
            // Add exit status to parent
            if context.pid == ppid {
                context.statuses.push(ContextStatus {
                    pid: pid,
                    status: status,
                });
                for status in statuses.iter() {
                    context.statuses.push(ContextStatus {
                        pid: status.pid,
                        status: status.status,
                    });
                }
            }

            // Move children to parent
            if context.ppid == pid {
                context.ppid = ppid;
            }
        }
    }

    loop {
        context_switch(false);
    }
}

pub unsafe fn do_sys_fpath(fd: usize, buf: *mut u8, len: usize) -> usize {
    let contexts = ::env().contexts.lock();
    if let Some(current) = contexts.get(context_i) {
        if let Some(resource) = current.get_file(fd) {
            let mut ret = 0;
            // TODO: Improve performance
            for b in resource.url().to_string().as_bytes().iter() {
                if ret < len {
                    ptr::write(buf.offset(ret as isize), *b);
                } else {
                    break;
                }
                ret += 1;
            }

            return ret;
        }
    }

    usize::MAX
}

pub unsafe fn do_sys_fsync(fd: usize) -> usize {
    let mut contexts = ::env().contexts.lock();
    if let Some(mut current) = contexts.get_mut(context_i) {
        if let Some(mut resource) = current.get_file_mut(fd) {
            if resource.sync() {
                return 0;
            }
        }
    }

    usize::MAX
}

pub unsafe fn do_sys_ftruncate(fd: usize, len: usize) -> usize {
    let mut contexts = ::env().contexts.lock();
    if let Some(mut current) = contexts.get_mut(context_i) {
        if let Some(mut resource) = current.get_file_mut(fd) {
            if resource.truncate(len) {
                return 0;
            }
        }
    }

    usize::MAX
}

pub unsafe fn do_sys_getpid() -> usize {
    let contexts = ::env().contexts.lock();
    if let Some(current) = contexts.get(context_i) {
        return current.pid;
    }

    usize::MAX
}

// TODO: link

pub unsafe fn do_sys_lseek(fd: usize, offset: isize, whence: usize) -> usize {
    let mut contexts = ::env().contexts.lock();
    if let Some(mut current) = contexts.get_mut(context_i) {
        if let Some(mut resource) = current.get_file_mut(fd) {
            match whence {
                SEEK_SET =>
                    if let Some(count) = resource.seek(ResourceSeek::Start(offset as usize)) {
                        return count;
                    },
                SEEK_CUR => if let Some(count) = resource.seek(ResourceSeek::Current(offset)) {
                    return count;
                },
                SEEK_END => if let Some(count) = resource.seek(ResourceSeek::End(offset)) {
                    return count;
                },
                _ => (),
            }
        }
    }

    usize::MAX
}

pub unsafe fn do_sys_mkdir(_: *const u8, _: usize) -> usize {
    // Implement body of do_sys_mkdir

    usize::MAX
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
    let mut contexts = ::env().contexts.lock();
    if let Some(mut current) = contexts.get_mut(context_i) {
        let path_string = current.canonicalize(str::from_utf8_unchecked(c_string_to_slice(path)));

        let resource_option = (::env()).open(&Url::from_string(path_string), flags);

        if let Some(resource) = resource_option {
            let fd = current.next_fd();

            (*current.files.get()).push(ContextFile {
                fd: fd,
                resource: resource,
            });

            return fd;
        }
    }

    usize::MAX
}

pub unsafe fn do_sys_read(fd: usize, buf: *mut u8, count: usize) -> usize {
    let mut contexts = ::env().contexts.lock();
    if let Some(mut current) = contexts.get_mut(context_i) {
        if let Some(resource) = current.get_file_mut(fd) {
            if let Some(count) = resource.read(slice::from_raw_parts_mut(buf, count)) {
                return count;
            }
        }
    }

    usize::MAX
}

pub unsafe fn do_sys_unlink(path: *const u8) -> usize {
    let contexts = ::env().contexts.lock();
    if let Some(current) = contexts.get(context_i) {
        let path_string = current.canonicalize(str::from_utf8_unchecked(c_string_to_slice(path)));

        if (::env()).unlink(&Url::from_string(path_string)) {
            return 0;
        }
    }

    usize::MAX
}

pub unsafe fn do_sys_waitpid(pid: isize, status: *mut usize, options: usize) -> usize {
    let mut ret = usize::MAX;

    loop {
        {
            let mut contexts = ::env().contexts.lock();
            if let Some(mut current) = contexts.get_mut(context_i) {
                let mut found = false;
                let mut i = 0;
                while i < current.statuses.len() {
                    if let Some(current_status) = current.statuses.get(i) {
                        if pid > 0 && pid as usize == current_status.pid {
                            // Specific child
                            found = true;
                        } else if pid == 0 {
                            // TODO Any child whose PGID is equal to this process
                        } else if pid == -1 {
                            // Any child
                            found = true;
                        } else {
                            // TODO Any child whose PGID is equal to abs(pid)
                        }
                    }
                    if found {
                        let current_status = current.statuses.remove(i);

                        ret = current_status.pid;
                        if status as usize > 0 {
                            ptr::write(status, current_status.status);
                        }

                        break;
                    } else {
                        i += 1;
                    }
                }
                if found {
                    break;
                }
            } else {
                break;
            }
        }

        context_switch(false);
    }

    ret
}

pub unsafe fn do_sys_write(fd: usize, buf: *const u8, count: usize) -> usize {
    let mut contexts = ::env().contexts.lock();
    if let Some(mut current) = contexts.get_mut(context_i) {
        if let Some(resource) = current.get_file_mut(fd) {
            if let Some(count) = resource.write(slice::from_raw_parts(buf, count)) {
                return count;
            }
        }
    }

    usize::MAX
}

pub unsafe fn do_sys_alloc(size: usize) -> usize {
    let mut ret = 0;

    let mut contexts = ::env().contexts.lock();
    if let Some(mut current) = contexts.get_mut(context_i) {
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

    ret
}

pub unsafe fn do_sys_realloc(ptr: usize, size: usize) -> usize {
    let mut ret = 0;

    let mut contexts = ::env().contexts.lock();
    if let Some(mut current) = contexts.get_mut(context_i) {
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

    ret
}

pub unsafe fn do_sys_realloc_inplace(ptr: usize, size: usize) -> usize {
    let mut ret = 0;

    let mut contexts = ::env().contexts.lock();
    if let Some(mut current) = contexts.get_mut(context_i) {
        current.unmap();
        if let Some(mut mem) = current.get_mem_mut(ptr) {
            mem.virtual_size = memory::realloc_inplace(mem.physical_address, size);
            ret = mem.virtual_size;
        }
        current.clean_mem();
        current.map();
    }

    ret
}

pub unsafe fn do_sys_unalloc(ptr: usize) {
    let mut contexts = ::env().contexts.lock();
    if let Some(mut current) = contexts.get_mut(context_i) {
        current.unmap();
        if let Some(mut mem) = current.get_mem_mut(ptr) {
            mem.virtual_size = 0;
        }
        current.clean_mem();
        current.map();
    }
}

pub unsafe fn syscall_handle(regs: &mut Regs) -> bool {
    match regs.ax {
        SYS_DEBUG => do_sys_debug(regs.bx as *const u8, regs.cx),
        // Linux
        SYS_BRK => regs.ax = do_sys_brk(regs.bx),
        SYS_CHDIR => regs.ax = do_sys_chdir(regs.bx as *const u8),
        SYS_CLONE => regs.ax = do_sys_clone(regs.bx),
        SYS_CLOSE => regs.ax = do_sys_close(regs.bx),
        SYS_CLOCK_GETTIME => regs.ax = do_sys_clock_gettime(regs.bx, regs.cx as *mut TimeSpec),
        SYS_DUP => regs.ax = do_sys_dup(regs.bx),
        SYS_EXECVE => regs.ax = do_sys_execve(regs.bx as *const u8, regs.cx as *const *const u8),
        SYS_SPAWNVE => regs.ax = do_sys_spawnve(regs.bx as *const u8, regs.cx as *const *const u8),
        SYS_EXIT => do_sys_exit(regs.bx),
        SYS_FPATH => regs.ax = do_sys_fpath(regs.bx, regs.cx as *mut u8, regs.dx),
        // TODO: fstat
        SYS_FSYNC => regs.ax = do_sys_fsync(regs.bx),
        SYS_FTRUNCATE => regs.ax = do_sys_ftruncate(regs.bx, regs.cx),
        SYS_GETPID => regs.ax = do_sys_getpid(),
        // TODO: link
        SYS_LSEEK => regs.ax = do_sys_lseek(regs.bx, regs.cx as isize, regs.dx),
        SYS_MKDIR => regs.ax = do_sys_mkdir(regs.bx as *const u8, regs.cx),
        SYS_NANOSLEEP =>
            regs.ax = do_sys_nanosleep(regs.bx as *const TimeSpec, regs.cx as *mut TimeSpec),
        SYS_OPEN => regs.ax = do_sys_open(regs.bx as *const u8, regs.cx), //regs.cx as isize, regs.dx as isize),
        SYS_READ => regs.ax = do_sys_read(regs.bx, regs.cx as *mut u8, regs.dx),
        SYS_UNLINK => regs.ax = do_sys_unlink(regs.bx as *const u8),
        SYS_WAITPID => regs.ax = do_sys_waitpid(regs.bx as isize, regs.cx as *mut usize, regs.dx),
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
