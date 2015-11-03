use collections::string::{String, ToString};
use collections::vec::Vec;

use core::ops::Deref;
use core::{ptr, slice, str, usize};

use scheduler::context::{context_clone, context_enabled, context_exit, context_switch, Context, ContextFile};
use common::debug;
use common::memory;
use scheduler;
use common::time::Duration;

use drivers::pio::*;

use programs::executor::execute;

use graphics::color::Color;
use graphics::size::Size;

use schemes::{Resource, ResourceSeek, URL};

use syscall::common::*;

pub unsafe fn do_sys_debug(byte: u8) {
    let reenable = scheduler::start_no_ints();

    if ::debug_display as usize > 0 {
        let display = &*::debug_display;
        display.rect(::debug_point, Size::new(8, 16), Color::new(0, 0, 0));
        if byte == 10 {
            ::debug_point.x = 0;
            ::debug_point.y += 16;
        } else if byte == 8 {
            //TODO: Fix up hack for backspace
            ::debug_point.x -= 8;
            if ::debug_point.x < 0 {
                ::debug_point.x = 0
            }
            display.rect(::debug_point, Size::new(8, 16), Color::new(0, 0, 0));
        } else {
            display.char(::debug_point, byte as char, Color::new(255, 255, 255));
            ::debug_point.x += 8;
        }
        if ::debug_point.x >= display.width as isize {
            ::debug_point.x = 0;
            ::debug_point.y += 16;
        }
        while ::debug_point.y + 16 > display.height as isize {
            display.scroll(16);
            ::debug_point.y -= 16;
        }
        display.rect(::debug_point,
                     Size::new(8, 16),
                     Color::new(255, 255, 255));
        ::debug_redraw = true;
        //If contexts disabled, probably booting up
        if ! context_enabled && ::debug_draw && ::debug_redraw {
            ::debug_redraw = false;
            display.flip();
        }
    }

    let serial_status = PIO8::new(0x3F8 + 5);
    while serial_status.read() & 0x20 == 0 {}

    let mut serial_data = PIO8::new(0x3F8);
    serial_data.write(byte);

    scheduler::end_no_ints(reenable);
}

pub unsafe fn do_sys_brk(addr: usize) -> usize {
    let mut ret = 0;

    let reenable = scheduler::start_no_ints();

    if let Some(mut current) = Context::current_mut() {
        current.unmap();

        if let Some(mut entry) = (*current.memory.get()).get_mut(0) {
            ret = entry.virtual_address + entry.virtual_size;

            if addr == 0 {
                //Get current break
            } else if addr >= entry.virtual_address {
                let request_size = addr - entry.virtual_address;
                let new_address = memory::realloc(entry.physical_address, request_size);
                if new_address > 0 {
                    ret = addr;

                    let new_size = memory::alloc_size(new_address);
                    entry.physical_address = new_address;
                    entry.virtual_size = new_size;
                } else {
                    debug::d("BRK: Realloc Failed\n");
                }
            } else {
                debug::d("BRK: Address not in correct space\n");
            }
        } else {
            debug::d("BRK: Memory not found\n");
        }

        current.map();
    } else {
        debug::d("BRK: Context not found\n");
    }

    scheduler::end_no_ints(reenable);

    ret
}

pub unsafe extern "cdecl" fn do_sys_chdir(path: *const u8) -> usize {
    let mut len = 0;
    while *path.offset(len as isize) > 0 {
        len += 1;
    }

    let mut ret = usize::MAX;

    let reenable = scheduler::start_no_ints();

    if let Some(current) = Context::current() {
        *current.cwd.get() = current.canonicalize(&str::from_utf8_unchecked(&slice::from_raw_parts(path, len)));
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
        contexts.push(Context::new(context_clone as usize, &context_clone_args));
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
            }else{
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

    if let Some(mut current) = Context::current_mut() {
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
            },
            CLOCK_MONOTONIC => {
                (*tp).tv_sec = ::clock_monotonic.secs;
                (*tp).tv_nsec = ::clock_monotonic.nanos;
                ret = 0;
            },
            _ => ()
        }
    }

    scheduler::end_no_ints(reenable);

    ret
}

pub unsafe fn do_sys_dup(fd: usize) -> usize {
    let mut ret = usize::MAX;

    let reenable = scheduler::start_no_ints();

    if let Some(mut current) = Context::current_mut() {
        let mut new_fd = current.next_fd();

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

//TODO: Cleanup
pub unsafe fn do_sys_execve(path: *const u8) -> usize {
    let mut ret = usize::MAX;


    let mut len = 0;
    while *path.offset(len as isize) > 0 {
        len += 1;
    }

    let path_str = String::from_utf8_unchecked(slice::from_raw_parts(path, len).to_vec());

    let reenable = scheduler::start_no_ints();

    if path_str.ends_with(".bin") {
        let path = URL::from_string(&path_str);
        let i = path_str.rfind('/').unwrap_or(0) + 1;
        let wd = URL::from_string(&path_str[ .. i].to_string());
        execute(&path,
                &wd,
                Vec::new());
        ret = 0;
    } else {
        for package in (*::session_ptr).packages.iter() {
            let mut accepted = false;
            for accept in package.accepts.iter() {
                if path_str.ends_with(&accept[1 ..]) {
                    accepted = true;
                    break;
                }
            }
            if accepted {
                let mut args: Vec<String> = Vec::new();
                args.push(path_str.clone());
                execute(&package.binary, &package.url, args);
                ret = 0;
                break;
            }
        }
    }

    scheduler::end_no_ints(reenable);

    ret
}

//TODO: Use argument
pub unsafe fn do_sys_exit(_: isize) {
    context_exit();
}

pub unsafe fn do_sys_fpath(fd: usize, buf: *mut u8, len: usize) -> usize {
    let mut ret = usize::MAX;

    let reenable = scheduler::start_no_ints();

    if let Some(current) = Context::current() {
        if let Some(resource) = current.get_file(fd) {
            scheduler::end_no_ints(reenable);

            ret = 0;
            //TODO: Improve performance
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

//TODO: link

pub unsafe fn do_sys_lseek(fd: usize, offset: isize, whence: usize) -> usize {
    let mut ret = usize::MAX;

    let reenable = scheduler::start_no_ints();

    if let Some(mut current) = Context::current_mut() {
        if let Some(mut resource) = current.get_file_mut(fd) {
            scheduler::end_no_ints(reenable);

            match whence {
                SEEK_SET => if let Some(count) = resource.seek(ResourceSeek::Start(offset as usize)) {
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

pub unsafe fn do_sys_nanosleep(req: *const TimeSpec, rem: *mut TimeSpec) -> usize{
    if req as usize > 0 {
        Duration::new((*req).tv_sec, (*req).tv_nsec).sleep();

        if rem as usize > 0 {
            (*rem).tv_sec = 0;
            (*rem).tv_nsec = 0;
        }

        0
    }else{
        usize::MAX
    }
}

pub unsafe fn do_sys_open(path: *const u8) -> usize {
    let mut len = 0;
    while *path.offset(len as isize) > 0 {
        len += 1;
    }

    let mut fd = usize::MAX;

    let reenable = scheduler::start_no_ints();

    if let Some(mut current) = Context::current_mut() {
        let path_str = current.canonicalize(str::from_utf8_unchecked(slice::from_raw_parts(path, len)));

        scheduler::end_no_ints(reenable);

        let resource_option = (*::session_ptr).open(&URL::from_string(&path_str));

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

//TODO: unlink

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

pub unsafe fn do_sys_yield() {
    context_switch(false);
}

pub unsafe fn do_sys_alloc(size: usize) -> usize {
    memory::alloc(size)
}

pub unsafe fn do_sys_realloc(ptr: usize, size: usize) -> usize {
    memory::realloc(ptr, size)
}

pub unsafe fn do_sys_realloc_inplace(ptr: usize, size: usize) -> usize {
    memory::realloc_inplace(ptr, size)
}

pub unsafe fn do_sys_unalloc(ptr: usize) {
    memory::unalloc(ptr)
}

pub unsafe fn syscall_handle(regs: &mut Regs) {
    match regs.ax {
        SYS_DEBUG => do_sys_debug(regs.bx as u8),
        // Linux
        SYS_BRK => regs.ax = do_sys_brk(regs.bx),
        SYS_CHDIR => regs.ax = do_sys_chdir(regs.bx as *const u8),
        SYS_CLONE => regs.ax = do_sys_clone(regs.bx),
        SYS_CLOSE => regs.ax = do_sys_close(regs.bx as usize),
        SYS_CLOCK_GETTIME => regs.ax = do_sys_clock_gettime(regs.bx, regs.cx as *mut TimeSpec),
        SYS_DUP => regs.ax = do_sys_dup(regs.bx),
        SYS_EXECVE => regs.ax = do_sys_execve(regs.bx as *const u8),
        SYS_EXIT => do_sys_exit(regs.bx as isize),
        SYS_FPATH => regs.ax = do_sys_fpath(regs.bx, regs.cx as *mut u8, regs.dx),
        //TODO: fstat
        SYS_FSYNC => regs.ax = do_sys_fsync(regs.bx),
        SYS_FTRUNCATE => regs.ax = do_sys_ftruncate(regs.bx, regs.cx),
        //TODO: link
        SYS_LSEEK => regs.ax = do_sys_lseek(regs.bx, regs.cx as isize, regs.dx as usize),
        SYS_NANOSLEEP => regs.ax = do_sys_nanosleep(regs.bx as *const TimeSpec, regs.cx as *mut TimeSpec),
        SYS_OPEN => regs.ax = do_sys_open(regs.bx as *const u8), //regs.cx as isize, regs.dx as isize),
        SYS_READ => regs.ax = do_sys_read(regs.bx, regs.cx as *mut u8, regs.dx),
        //TODO: unlink
        SYS_WRITE => regs.ax = do_sys_write(regs.bx, regs.cx as *mut u8, regs.dx),
        SYS_YIELD => do_sys_yield(),

        // Rust Memory
        SYS_ALLOC => regs.ax = do_sys_alloc(regs.bx),
        SYS_REALLOC => regs.ax = do_sys_realloc(regs.bx, regs.cx),
        SYS_REALLOC_INPLACE => regs.ax = do_sys_realloc_inplace(regs.bx, regs.cx),
        SYS_UNALLOC => do_sys_unalloc(regs.bx),

        _ => {
            debug::d("Unknown Syscall: ");
            debug::dd(regs.ax as usize);
            debug::d(", ");
            debug::dh(regs.bx as usize);
            debug::d(", ");
            debug::dh(regs.cx as usize);
            debug::d(", ");
            debug::dh(regs.dx as usize);
            debug::dl();

            regs.ax = usize::MAX;
        }
    }
}
