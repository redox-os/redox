use alloc::arc::Arc;

use arch::context::{context_clone, context_switch, Context, ContextMemory, ContextFile, ContextStatus};
use arch::memory;
use arch::regs::Regs;

use collections::string::{ToString};
use collections::vec::Vec;

use core::ops::Deref;
use core::{mem, ptr, slice, str, usize};

use common::time::Duration;

use drivers::io::{Io, Pio};

use schemes::{Resource, ResourceSeek, Url};
use schemes::pipe::{PipeRead, PipeWrite};

use sync::Intex;

use syscall::*;

use super::execute::execute;

/// Helper function for handling C strings, please do not copy it or make it pub or change it
fn c_string_to_slice<'a>(ptr: *const u8) -> &'a [u8] {
    if ptr > 0 as *const u8 {
        let mut len = 0;
        while unsafe { ptr::read(ptr.offset(len as isize)) } > 0 {
            len += 1;
        }

        unsafe { slice::from_raw_parts(ptr, len) }
    } else {
        &[]
    }
}
/// Helper function for handling C strings, please do not copy it or make it pub or change it
fn c_array_to_slice<'a>(ptr: *const *const u8) -> &'a [*const u8] {
    if ptr > 0 as *const *const u8 {
        let mut len = 0;
        while unsafe { ptr::read(ptr.offset(len as isize)) } > 0 as *const u8 {
            len += 1;
        }

        unsafe { slice::from_raw_parts(ptr, len) }
    } else {
        &[]
    }
}

pub fn do_sys_debug(ptr: *const u8, len: usize) {
    let bytes = unsafe { slice::from_raw_parts(ptr, len) };

    if unsafe { ::ENV_PTR.is_some() } {
        ::env().console.lock().write(bytes);
    } else {
        let serial_status = Pio::<u8>::new(0x3F8 + 5);
        let mut serial_data = Pio::<u8>::new(0x3F8);

        for byte in bytes.iter() {
            while !serial_status.readf(0x20) {}
            serial_data.write(*byte);

            if *byte == 8 {
                while !serial_status.readf(0x20) {}
                serial_data.write(0x20);

                while !serial_status.readf(0x20) {}
                serial_data.write(8);
            }
        }
    }
}

pub fn do_sys_brk(addr: usize) -> usize {
    let mut ret = 0;

    let mut contexts = ::env().contexts.lock();
    if let Some(mut current) = contexts.current_mut() {
        unsafe {
            current.unmap();
        }

        ret = unsafe { current.next_mem() };

        // TODO: Make this smarter, currently it attempt to resize the entire data segment
        if let Some(mut mem) = unsafe { (*current.memory.get()).last_mut() } {
            if mem.writeable && mem.allocated {
                if addr >= mem.virtual_address {
                    let size = addr - mem.virtual_address;
                    let physical_address = unsafe { memory::realloc(mem.physical_address, size) };
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
                debug!("BRK: End segment not writeable or allocated\n");
            }
        } else {
            debug!("BRK: No segments\n")
        }

        unsafe {
            current.clean_mem();
            current.map();
        }
    } else {
        debug!("BRK: Context not found\n");
    }

    ret
}

pub extern "cdecl" fn do_sys_chdir(path: *const u8) -> usize {
    let contexts = ::env().contexts.lock();
    Error::mux(if let Some(current) = contexts.current() {
        unsafe {
            *current.cwd.get() =
                current.canonicalize(&str::from_utf8_unchecked(&c_string_to_slice(path)));
        }
        Ok(0)
    } else {
        Err(Error::new(ESRCH))
    })
}

#[cold]
#[inline(never)]
pub fn do_sys_clone(flags: usize) -> usize {
    let mut clone_pid = usize::MAX;
    let mut mem_count = 0;

    {
        let mut contexts = ::env().contexts.lock();

        let child_option = if let Some(parent) = contexts.current() {
            clone_pid = unsafe { Context::next_pid() };
            mem_count = Arc::strong_count(&parent.memory);

            let parent_ptr: *const Context = parent.deref();

            let mut context_clone_args: Vec<usize> = Vec::new();
            context_clone_args.push(clone_pid);
            context_clone_args.push(flags);
            context_clone_args.push(parent_ptr as usize);
            context_clone_args.push(0); //Return address, 0 catches bad code

            Some(unsafe {
                Context::new(format!("kclone {}", parent.name),
                             context_clone as usize,
                             &context_clone_args)
            })
        } else {
            None
        };

        if let Some(child) = child_option {
            unsafe { contexts.push(child) };
        }
    }

    unsafe {
        context_switch(false);
    }

    Error::mux(if clone_pid != usize::MAX {
        let contexts = ::env().contexts.lock();
        if let Some(current) = contexts.current() {
            if current.pid == clone_pid {
                Ok(0)
            } else {
                if flags & CLONE_VFORK == CLONE_VFORK {
                    while Arc::strong_count(&current.memory) > mem_count {
                        unsafe {
                            context_switch(false);
                        }
                    }
                }
                Ok(clone_pid)
            }
        } else {
            Err(Error::new(ESRCH))
        }
    } else {
        Err(Error::new(ESRCH))
    })
}

pub fn do_sys_close(fd: usize) -> usize {
    let contexts = ::env().contexts.lock();
    Error::mux(if let Some(current) = contexts.current() {
        let mut ret = Err(Error::new(EBADF));

        for i in 0..unsafe { (*current.files.get()).len() } {
            let mut remove = false;
            if let Some(file) = unsafe { (*current.files.get()).get(i) } {
                if file.fd == fd {
                    remove = true;
                }
            }

            if remove {
                if i < unsafe { (*current.files.get()).len() } {
                    drop(unsafe { (*current.files.get()).remove(i) });

                    ret = Ok(0);
                }

                break;
            }
        }

        ret
    } else {
        Err(Error::new(ESRCH))
    })
}

pub fn do_sys_clock_gettime(clock: usize, tp: *mut TimeSpec) -> usize {
    let _intex = Intex::static_lock();

    Error::mux(if tp as usize > 0 {
        match clock {
            CLOCK_REALTIME => {
                let clock_realtime = ::env().clock_realtime.lock();
                unsafe {
                    (*tp).tv_sec = clock_realtime.secs;
                    (*tp).tv_nsec = clock_realtime.nanos;
                }
                Ok(0)
            }
            CLOCK_MONOTONIC => {
                let clock_monotonic = ::env().clock_monotonic.lock();
                unsafe {
                    (*tp).tv_sec = clock_monotonic.secs;
                    (*tp).tv_nsec = clock_monotonic.nanos;
                }
                Ok(0)
            }
            _ => Err(Error::new(EINVAL)),
        }
    } else {
        Err(Error::new(EFAULT))
    })
}

pub fn do_sys_dup(fd: usize) -> usize {
    let contexts = ::env().contexts.lock();
    Error::mux(if let Some(current) = contexts.current() {
        if let Some(resource) = unsafe { current.get_file(fd) } {
            match resource.dup() {
                Ok(new_resource) => {
                    let new_fd = unsafe { current.next_fd() };

                    unsafe {
                        (*current.files.get()).push(ContextFile {
                            fd: new_fd,
                            resource: new_resource,
                        });
                    }

                    Ok(new_fd)
                }
                Err(err) => Err(err),
            }
        } else {
            Err(Error::new(EBADF))
        }
    } else {
        Err(Error::new(ESRCH))
    })
}

pub fn do_sys_execve(path: *const u8, args: *const *const u8) -> usize {
    let mut args_vec = Vec::new();
    args_vec.push(unsafe { str::from_utf8_unchecked(c_string_to_slice(path)) }.to_string());
    for arg in c_array_to_slice(args) {
        args_vec.push(unsafe { str::from_utf8_unchecked(c_string_to_slice(*arg)) }.to_string());
    }

    Error::mux(execute(args_vec))
}

/// Exit context
///
/// Unsafe due to interrupt disabling and raw pointers
pub fn do_sys_exit(status: usize) {
    {
        let mut contexts = ::env().contexts.lock();

        let mut statuses = Vec::new();
        let (pid, ppid) = {
            if let Some(mut current) = contexts.current_mut() {
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
        unsafe {
            context_switch(false);
        }
    }
}

pub fn do_sys_fpath(fd: usize, buf: *mut u8, len: usize) -> usize {
    let contexts = ::env().contexts.lock();
    Error::mux(if let Some(current) = contexts.current() {
        if let Some(resource) = unsafe { current.get_file(fd) } {
            let mut ret = 0;
            // TODO: Improve performance
            for b in resource.url().to_string().as_bytes().iter() {
                if ret < len {
                    unsafe {
                        ptr::write(buf.offset(ret as isize), *b);
                    }
                } else {
                    break;
                }
                ret += 1;
            }

            Ok(ret)
        } else {
            Err(Error::new(EBADF))
        }
    } else {
        Err(Error::new(ESRCH))
    })
}

pub fn do_sys_fsync(fd: usize) -> usize {
    let mut contexts = ::env().contexts.lock();
    Error::mux(if let Some(mut current) = contexts.current_mut() {
        if let Some(mut resource) = unsafe { current.get_file_mut(fd) } {
            match resource.sync() {
                Ok(_) => Ok(0),
                Err(err) => Err(err),
            }
        } else {
            Err(Error::new(EBADF))
        }
    } else {
        Err(Error::new(ESRCH))
    })
}

pub fn do_sys_ftruncate(fd: usize, len: usize) -> usize {
    let mut contexts = ::env().contexts.lock();
    Error::mux(if let Some(mut current) = contexts.current_mut() {
        if let Some(mut resource) = unsafe { current.get_file_mut(fd) } {
            match resource.truncate(len) {
                Ok(_) => Ok(0),
                Err(err) => Err(err),
            }
        } else {
            Err(Error::new(EBADF))
        }
    } else {
        Err(Error::new(ESRCH))
    })
}

pub fn do_sys_getpid() -> usize {
    let contexts = ::env().contexts.lock();
    Error::mux(if let Some(current) = contexts.current() {
        Ok(current.pid)
    } else {
        Err(Error::new(ESRCH))
    })
}

// TODO: link

pub fn do_sys_lseek(fd: usize, offset: isize, whence: usize) -> usize {
    let mut contexts = ::env().contexts.lock();
    Error::mux(if let Some(mut current) = contexts.current_mut() {
        if let Some(mut resource) = unsafe { current.get_file_mut(fd) } {
            match whence {
                SEEK_SET => resource.seek(ResourceSeek::Start(offset as usize)),
                SEEK_CUR => resource.seek(ResourceSeek::Current(offset)),
                SEEK_END => resource.seek(ResourceSeek::End(offset)),
                _ => Err(Error::new(EINVAL)),
            }
        } else {
            Err(Error::new(EBADF))
        }
    } else {
        Err(Error::new(ESRCH))
    })
}

pub fn do_sys_mkdir(_: *const u8, _: usize) -> usize {
    // Implement body of do_sys_mkdir

    usize::MAX
}

pub fn do_sys_nanosleep(req: *const TimeSpec, rem: *mut TimeSpec) -> usize {
    Error::mux(if req as usize > 0 {
        Duration::new(unsafe { (*req).tv_sec }, unsafe { (*req).tv_nsec }).sleep();

        if rem as usize > 0 {
            unsafe {
                (*rem).tv_sec = 0;
            }
            unsafe {
                (*rem).tv_nsec = 0;
            }
        }

        Ok(0)
    } else {
        Err(Error::new(EFAULT))
    })
}

pub fn do_sys_open(path: *const u8, flags: usize) -> usize {
    let contexts = ::env().contexts.lock();
    Error::mux(if let Some(current) = contexts.current() {
        let path_string = unsafe {
            current.canonicalize(str::from_utf8_unchecked(c_string_to_slice(path)))
        };

        match (::env()).open(&Url::from_string(path_string), flags) {
            Ok(resource) => {
                let fd = unsafe { current.next_fd() };

                unsafe {
                    (*current.files.get()).push(ContextFile {
                        fd: fd,
                        resource: resource,
                    });
                }

                Ok(fd)
            }
            Err(err) => Err(err),
        }
    } else {
        Err(Error::new(ESRCH))
    })
}

pub fn do_sys_pipe2(fds: *mut usize, _flags: usize) -> usize {
    let contexts = ::env().contexts.lock();
    Error::mux(if let Some(current) = contexts.current() {
        if fds as usize > 0 {
            let read = box PipeRead::new();
            let write = box PipeWrite::new(&read);

            unsafe {
                *fds.offset(0) = current.next_fd();
                (*current.files.get()).push(ContextFile {
                    fd: *fds.offset(0),
                    resource: read,
                });

                *fds.offset(1) = current.next_fd();
                (*current.files.get()).push(ContextFile {
                    fd: *fds.offset(1),
                    resource: write,
                });
            }

            Ok(0)
        } else {
            Err(Error::new(EFAULT))
        }
    } else {
        Err(Error::new(ESRCH))
    })
}

pub fn do_sys_read(fd: usize, buf: *mut u8, count: usize) -> usize {
    let mut contexts = ::env().contexts.lock();
    Error::mux(if let Some(mut current) = contexts.current_mut() {
        if let Some(resource) = unsafe { current.get_file_mut(fd) } {
            resource.read(unsafe { slice::from_raw_parts_mut(buf, count) })
        } else {
            Err(Error::new(EBADF))
        }
    } else {
        Err(Error::new(ESRCH))
    })
}

pub fn do_sys_unlink(path: *const u8) -> usize {
    let contexts = ::env().contexts.lock();
    Error::mux(if let Some(current) = contexts.current() {
        let path_string = unsafe {
            current.canonicalize(str::from_utf8_unchecked(c_string_to_slice(path)))
        };

        match (::env()).unlink(&Url::from_string(path_string)) {
            Ok(_) => Ok(0),
            Err(err) => Err(err),
        }
    } else {
        Err(Error::new(ESRCH))
    })
}

pub fn do_sys_waitpid(pid: isize, status: *mut usize, _options: usize) -> usize {
    let mut ret = Err(Error::new(ECHILD));

    loop {
        {
            let mut contexts = ::env().contexts.lock();
            if let Some(mut current) = contexts.current_mut() {
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

                        ret = Ok(current_status.pid);
                        if status as usize > 0 {
                            unsafe {
                                ptr::write(status, current_status.status);
                            }
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

        unsafe {
            context_switch(false);
        }
    }

    Error::mux(ret)
}

pub fn do_sys_write(fd: usize, buf: *const u8, count: usize) -> usize {
    let mut contexts = ::env().contexts.lock();
    Error::mux(if let Some(mut current) = contexts.current_mut() {
        if let Some(resource) = unsafe { current.get_file_mut(fd) } {
            resource.write(unsafe { slice::from_raw_parts(buf, count) })
        } else {
            Err(Error::new(EBADF))
        }
    } else {
        Err(Error::new(ESRCH))
    })
}

pub fn do_sys_alloc(size: usize) -> usize {
    let mut ret = 0;

    let mut contexts = ::env().contexts.lock();
    if let Some(current) = contexts.current() {
        let physical_address = unsafe { memory::alloc(size) };
        if physical_address > 0 {
            let mut mem = ContextMemory {
                physical_address: physical_address,
                virtual_address: unsafe { current.next_mem() },
                virtual_size: size,
                writeable: true,
                allocated: true,
            };

            ret = mem.virtual_address;

            unsafe {
                mem.map();
                (*current.memory.get()).push(mem);
            }
        }
    }

    ret
}

pub fn do_sys_realloc(ptr: usize, size: usize) -> usize {
    let mut ret = 0;

    let mut contexts = ::env().contexts.lock();
    if let Some(mut current) = contexts.current_mut() {
        if let Some(mut mem) = unsafe { current.get_mem_mut(ptr) } {
            unsafe { mem.unmap(); }

            let physical_address = unsafe { memory::realloc(mem.physical_address, size) };
            if physical_address > 0 {
                mem.physical_address = physical_address;
                mem.virtual_size = size;
                ret = mem.virtual_address;
            } else {
                mem.virtual_size = 0;
            }

            unsafe { mem.map(); }
        }
        unsafe { current.clean_mem(); }
    }

    ret
}

pub fn do_sys_realloc_inplace(ptr: usize, size: usize) -> usize {
    let mut ret = 0;

    let mut contexts = ::env().contexts.lock();
    if let Some(mut current) = contexts.current_mut() {
        if let Some(mut mem) = unsafe { current.get_mem_mut(ptr) } {
            unsafe { mem.unmap(); }

            mem.virtual_size = unsafe { memory::realloc_inplace(mem.physical_address, size) };
            ret = mem.virtual_size;

            unsafe {mem.map(); }
        }
        unsafe { current.clean_mem(); }
    }

    ret
}

pub fn do_sys_unalloc(ptr: usize) {
    let mut contexts = ::env().contexts.lock();
    if let Some(mut current) = contexts.current_mut() {
        if let Some(mut mem) = unsafe { current.get_mem_mut(ptr) } {
            unsafe { mem.unmap() };

            mem.virtual_size = 0;
        }
        unsafe { current.clean_mem(); }
    }
}

pub fn syscall_handle(regs: &mut Regs) {
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
        SYS_PIPE2 => regs.ax = do_sys_pipe2(regs.bx as *mut usize, regs.cx),
        SYS_READ => regs.ax = do_sys_read(regs.bx, regs.cx as *mut u8, regs.dx),
        SYS_UNLINK => regs.ax = do_sys_unlink(regs.bx as *const u8),
        SYS_WAITPID => regs.ax = do_sys_waitpid(regs.bx as isize, regs.cx as *mut usize, regs.dx),
        SYS_WRITE => regs.ax = do_sys_write(regs.bx, regs.cx as *mut u8, regs.dx),
        SYS_YIELD => unsafe { context_switch(false) },

        // Rust Memory
        SYS_ALLOC => regs.ax = do_sys_alloc(regs.bx),
        SYS_REALLOC => regs.ax = do_sys_realloc(regs.bx, regs.cx),
        SYS_REALLOC_INPLACE => regs.ax = do_sys_realloc_inplace(regs.bx, regs.cx),
        SYS_UNALLOC => do_sys_unalloc(regs.bx),

        _ => regs.ax = Error::mux(Err(Error::new(ENOSYS))),
    }
}
