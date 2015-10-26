use alloc::boxed::Box;

use collections::string::{String, ToString};
use collections::vec::Vec;

use core::{ptr, slice, usize};

use common::context::*;
use common::debug;
use common::memory;
use common::scheduler;

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
        //If interrupts disabled, probably booting up
        if !reenable && ::debug_draw && ::debug_redraw {
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

    let contexts = &mut *contexts_ptr;
    if context_enabled && context_i > 1 {
        if let Some(mut current) = contexts.get_mut(context_i) {
            current.unmap();

            if let Some(mut entry) = current.memory.get_mut(0) {
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
    } else {
        debug::d("BRK: Contexts disabled\n");
    }

    scheduler::end_no_ints(reenable);

    ret
}

//TODO: chdir

pub unsafe fn do_sys_close(fd: usize) -> usize {
    let mut ret = usize::MAX;

    let reenable = scheduler::start_no_ints();

    let contexts = &mut *contexts_ptr;
    if let Some(mut current) = contexts.get_mut(context_i) {
        for i in 0..current.files.len() {
            let mut remove = false;
            if let Some(file) = current.files.get(i) {
                if file.fd == fd {
                    remove = true;
                }
            }

            if remove {
                if i < current.files.len() {
                    let file = current.files.remove(i);

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

pub unsafe fn do_sys_dup(fd: usize) -> usize {
    let mut ret = usize::MAX;

    let reenable = scheduler::start_no_ints();

    let contexts = &mut *contexts_ptr;
    if let Some(mut current) = contexts.get_mut(context_i) {
        let mut resource_option: Option<Box<Resource>> = None;
        let mut new_fd = 0;
        for file in current.files.iter() {
            if file.fd == fd {
                resource_option = file.resource.dup();
            }
            if file.fd >= new_fd {
                new_fd = file.fd + 1;
            }
        }

        if let Some(resource) = resource_option {
            ret = new_fd;
            current.files.push(ContextFile {
                fd: new_fd,
                resource: resource,
            });
        }
    }

    scheduler::end_no_ints(reenable);

    ret
}

pub unsafe fn do_sys_execve(path: *const u8) -> usize {
    let mut ret = usize::MAX;


    let mut len = 0;
    while *path.offset(len as isize) > 0 {
        len += 1;
    }

    let path_str = String::from_utf8_unchecked(slice::from_raw_parts(path, len).to_vec());

    let reenable = scheduler::start_no_ints();

    if path_str.ends_with(".bin") {
        execute(&URL::from_string(&path_str),
                &URL::new(),
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

pub unsafe fn do_sys_exit(status: isize) {
    context_exit();
}

pub unsafe fn do_sys_fork() -> usize {
    let mut ret = usize::MAX;

    let reenable = scheduler::start_no_ints();

    let parent_i = context_i;

    let contexts = &mut *contexts_ptr;

    let mut context_fork_args: Vec<usize> = Vec::new();
    context_fork_args.push(parent_i);
    context_fork_args.push(context_exit as usize);

    contexts.push(Context::new(context_fork as usize, &context_fork_args));

    context_switch(true);

    if context_i == parent_i {
        ret = 0;
    }else{
        ret = context_i;
    }

    scheduler::end_no_ints(reenable);

    ret
}

pub unsafe fn do_sys_fpath(fd: usize, buf: *mut u8, len: usize) -> usize {
    let mut ret = usize::MAX;

    let reenable = scheduler::start_no_ints();

    let contexts = &*contexts_ptr;
    if let Some(current) = contexts.get(context_i) {
        for i in 0..current.files.len() {
            if let Some(file) = current.files.get(i) {
                if file.fd == fd {
                    scheduler::end_no_ints(reenable);

                    ret = 0;
                    //TODO: Improve performance
                    for b in file.resource.url().to_string().as_bytes().iter() {
                        if ret < len {
                            ptr::write(buf.offset(ret as isize), *b);
                        } else {
                            break;
                        }
                        ret += 1;
                    }

                    scheduler::start_no_ints();

                    break;
                }
            }
        }
    }

    scheduler::end_no_ints(reenable);

    ret
}

pub unsafe fn do_sys_fsync(fd: usize) -> usize {
    let mut ret = usize::MAX;

    let reenable = scheduler::start_no_ints();

    let contexts = &mut *contexts_ptr;
    if let Some(mut current) = contexts.get_mut(context_i) {
        for i in 0..current.files.len() {
            if let Some(mut file) = current.files.get_mut(i) {
                if file.fd == fd {
                    scheduler::end_no_ints(reenable);

                    if file.resource.sync() {
                        ret = 0;
                    }

                    scheduler::start_no_ints();

                    break;
                }
            }
        }
    }

    scheduler::end_no_ints(reenable);

    ret
}

#[repr(packed)]
pub struct TV {
    pub tv_sec: i64,
    pub tv_usec: i32,
}

pub unsafe fn do_sys_gettimeofday(tv: *mut TV) -> usize {
    let reenable = scheduler::start_no_ints();

    if tv as usize > 0 {
        (*tv).tv_sec = ::clock_realtime.secs;
        (*tv).tv_usec = ::clock_realtime.nanos/1000;
    }

    scheduler::end_no_ints(reenable);

    0
}

//TODO: link

pub unsafe fn do_sys_lseek(fd: usize, offset: isize, whence: usize) -> usize {
    let mut ret = usize::MAX;

    let reenable = scheduler::start_no_ints();

    let contexts = &mut *contexts_ptr;
    if let Some(mut current) = contexts.get_mut(context_i) {
        for mut file in current.files.iter_mut() {
            if file.fd == fd {
                scheduler::end_no_ints(reenable);

                match whence {
                    0 => if let Some(count) = file.resource.seek(ResourceSeek::Start(offset as usize)) {
                        ret = count;
                    },
                    1 => if let Some(count) = file.resource.seek(ResourceSeek::Current(offset)) {
                        ret = count;
                    },
                    2 =>
                        if let Some(count) = file.resource.seek(ResourceSeek::End(offset)) {
                        ret = count;
                    },
                    _ => (),
                }

                scheduler::start_no_ints();

                break;
            }
        }
    }

    scheduler::end_no_ints(reenable);

    ret
}

pub unsafe fn do_sys_open(path: *const u8, flags: isize, mode: isize) -> usize {
    let mut len = 0;
    while *path.offset(len as isize) > 0 {
        len += 1;
    }

    let mut path_str = String::from_utf8_unchecked(slice::from_raw_parts(path, len).to_vec());

    //TODO: Handle more path derivatives

    if path_str.find(':').is_none() {
        let reenable = scheduler::start_no_ints();

        let contexts = &*contexts_ptr;
        if let Some(current) = contexts.get(context_i) {
            if path_str.starts_with('/') {
                let i = current.cwd.find(':').unwrap_or(0) + 1;
                path_str = current.cwd[.. i].to_string() + &path_str;
            }else{
                path_str = current.cwd.clone() + &path_str;
            }
        }

        scheduler::end_no_ints(reenable);
    }

    let mut fd = usize::MAX;

    if let Some(resource) = (*::session_ptr).open(&URL::from_string(&path_str)) {
        let reenable = scheduler::start_no_ints();

        let contexts = &mut *contexts_ptr;
        if let Some(mut current) = contexts.get_mut(context_i) {
            fd = 0;
            for file in current.files.iter() {
                if file.fd >= fd {
                    fd = file.fd + 1;
                }
            }

            current.files.push(ContextFile {
                fd: fd,
                resource: resource,
            });
        }

        scheduler::end_no_ints(reenable);
    }

    fd
}

pub unsafe fn do_sys_read(fd: usize, buf: *mut u8, count: usize) -> usize {
    let mut ret = usize::MAX;

    let reenable = scheduler::start_no_ints();

    let contexts = &mut *contexts_ptr;
    if let Some(mut current) = contexts.get_mut(context_i) {
        for mut file in current.files.iter_mut() {
            if file.fd == fd {
                scheduler::end_no_ints(reenable);

                if let Some(count) = file.resource.read(slice::from_raw_parts_mut(buf, count)) {
                    ret = count;
                }

                scheduler::start_no_ints();

                break;
            }
        }
    }

    scheduler::end_no_ints(reenable);

    ret
}

//TODO: unlink

pub unsafe fn do_sys_write(fd: usize, buf: *const u8, count: usize) -> usize {
    let mut ret = usize::MAX;

    let reenable = scheduler::start_no_ints();

    let contexts = &mut *contexts_ptr;
    if let Some(mut current) = contexts.get_mut(context_i) {
        for mut file in current.files.iter_mut() {
            if file.fd == fd {
                scheduler::end_no_ints(reenable);

                if let Some(count) = file.resource.write(slice::from_raw_parts(buf, count)) {
                    ret = count;
                }

                scheduler::start_no_ints();

                break;
            }
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

pub unsafe fn syscall_handle(mut eax: usize, ebx: usize, ecx: usize, edx: usize) -> usize {
    match eax {
        SYS_DEBUG => do_sys_debug(ebx as u8),
        // Linux
        SYS_BRK => eax = do_sys_brk(ebx),
        //TODO: chdir
        SYS_CLOSE => eax = do_sys_close(ebx as usize),
        SYS_DUP => eax = do_sys_dup(ebx),
        SYS_EXECVE => eax = do_sys_execve(ebx as *const u8),
        SYS_EXIT => do_sys_exit(ebx as isize),
        SYS_FORK => eax = do_sys_fork(),
        SYS_FPATH => eax = do_sys_fpath(ebx, ecx as *mut u8, edx),
        //TODO: fstat
        SYS_FSYNC => eax = do_sys_fsync(ebx),
        SYS_GETTIMEOFDAY => eax = do_sys_gettimeofday(ebx as *mut TV),
        //TODO: link
        SYS_LSEEK => eax = do_sys_lseek(ebx, ecx as isize, edx as usize),
        SYS_OPEN => eax = do_sys_open(ebx as *const u8, ecx as isize, edx as isize),
        SYS_READ => eax = do_sys_read(ebx, ecx as *mut u8, edx),
        //TODO: unlink
        SYS_WRITE => eax = do_sys_write(ebx, ecx as *mut u8, edx),
        SYS_YIELD => do_sys_yield(),

        // Rust Memory
        SYS_ALLOC => eax = do_sys_alloc(ebx),
        SYS_REALLOC => eax = do_sys_realloc(ebx, ecx),
        SYS_REALLOC_INPLACE => eax = do_sys_realloc_inplace(ebx, ecx),
        SYS_UNALLOC => do_sys_unalloc(ebx),

        _ => {
            debug::d("Unknown Syscall: ");
            debug::dd(eax as usize);
            debug::d(", ");
            debug::dh(ebx as usize);
            debug::d(", ");
            debug::dh(ecx as usize);
            debug::d(", ");
            debug::dh(edx as usize);
            debug::dl();
        }
    }

    eax
}
