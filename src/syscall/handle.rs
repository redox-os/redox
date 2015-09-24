use alloc::boxed::*;

use core::cmp::max;
use core::cmp::min;
use core::mem::size_of;
use core::ptr;
use core::slice;

use common::context::*;
use common::debug::*;
use common::event::*;
use common::memory::*;
use common::pio::*;
use common::resource::*;
use common::scheduler::*;
use common::string::*;
use common::time::*;
use common::vec::*;

use graphics::color::*;
use graphics::window::*;

use syscall::common::*;

pub unsafe fn do_sys_debug(byte: u8){
    let reenable = start_no_ints();

    if ::debug_display as usize > 0 {
        let display = &*(*::debug_display);
        if byte == 10 {
            ::debug_point.x = 0;
            ::debug_point.y += 16;
            ::debug_redraw = true;
        }else{
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
        //If interrupts disabled, probably booting up
        if !reenable && ::debug_draw && ::debug_redraw {
            ::debug_redraw = false;
            display.flip();
        }
    }

    outb(0x3F8, byte);

    end_no_ints(reenable);
}

pub unsafe fn do_sys_exit(status: isize) {
    context_exit();
}

pub unsafe fn do_sys_read(fd: usize, buf: *mut u8, count: usize) -> usize {
    d("Read ");
    dh(fd);
    dl();
    if fd == 0 {
        //TODO: Read stdin
        return 0xFFFFFFFF;
    }else if alloc_size(fd) >= size_of::<Box<Resource>>() {
        let resource_ptr: *mut Box<Resource> = fd as *mut Box<Resource>;
        match (*resource_ptr).read(slice::from_raw_parts_mut(buf, count)) {
            Option::Some(count) => return count,
            Option::None => return 0xFFFFFFFF
        }
    }else{
        return 0xFFFFFFFF;
    }
}

//TODO: Remove
pub unsafe fn do_sys_read_to_end(fd: usize, vec: *mut Vec<u8>) -> usize {
    d("Read To End ");
    dh(fd);
    dl();
    if fd == 0 {
        //TODO: Read stdin
        return 0xFFFFFFFF;
    }else if alloc_size(fd) >= size_of::<Box<Resource>>() {
        let resource_ptr: *mut Box<Resource> = fd as *mut Box<Resource>;
        match (*resource_ptr).read_to_end(&mut *vec) {
            Option::Some(count) => return count,
            Option::None => return 0xFFFFFFFF
        }
    }else{
        return 0xFFFFFFFF;
    }
}

pub unsafe fn do_sys_write(fd: usize, buf: *const u8, count: usize) -> usize {
    d("Write ");
    dh(fd);
    dl();
    if fd == 1 || fd == 2 {
        for i in 0..count as isize {
            do_sys_debug(*buf.offset(i));
        }
        return count;
    }else if alloc_size(fd) >= size_of::<Box<Resource>>() {
        let resource_ptr: *mut Box<Resource> = fd as *mut Box<Resource>;
        match (*resource_ptr).write(slice::from_raw_parts(buf, count)) {
            Option::Some(count) => return count,
            Option::None => return 0xFFFFFFFF
        }
    }else{
        return 0xFFFFFFFF
    }
}

#[inline(never)]
pub unsafe fn do_sys_open(path: *const u8, flags: isize, mode: isize) -> usize {
    let resource_ptr: *mut Box<Resource> = alloc_type();
    let path_str = String::from_c_str(path);
    ptr::write(resource_ptr, (*::session_ptr).open(&URL::from_string(&path_str)));
    d("Open ");
    path_str.d();
    d(" ");
    dh(resource_ptr as usize);
    dl();
    return resource_ptr as usize;
}

#[inline(never)]
pub unsafe fn do_sys_close(fd: usize) -> usize {
    d("Close ");
    dh(fd);
    dl();
    if fd == 0 || fd == 1 || fd == 2 {
        return 0;
    }else if alloc_size(fd) >= size_of::<Box<Resource>>() {
        let resource_ptr: *mut Box<Resource> = fd as *mut Box<Resource>;
        drop(ptr::read(resource_ptr));
        unalloc(resource_ptr as usize);
        return 0;
    }else{
        return 0xFFFFFFFF;
    }
}

#[inline(never)]
pub unsafe fn do_sys_brk(addr: usize) -> usize {
    let mut ret = 0;

    let reenable = start_no_ints();

    let contexts = &*(*contexts_ptr);
    if context_enabled && context_i > 1 {
        if let Option::Some(mut current) = contexts.get(context_i) {
            current.unmap();

            if let Option::Some(mut entry) = current.memory.get(0) {
                ret = entry.virtual_address + entry.virtual_size;

                if addr == 0 {
                    //Get current break
                }else if addr >= entry.virtual_address {
                    let request_size = addr - entry.virtual_address;
                    let new_address = realloc(entry.physical_address, request_size);
                    if new_address > 0 {
                        ret = addr;

                        let new_size = alloc_size(new_address);
                        entry.physical_address = new_address;
                        entry.virtual_size = new_size;
                    }else{
                        d("BRK: Realloc Failed\n");
                    }
                }else{
                    d("BRK: Address not in correct space\n");
                }
            }else{
                d("BRK: Memory not found\n");
            }

            current.map();
        }else{
            d("BRK: Context not found\n");
        }
    }else{
        d("BRK: Contexts disabled\n");
    }

    end_no_ints(reenable);

    return ret;
}

#[inline(never)]
pub unsafe fn syscall_handle(mut eax: u32, ebx: u32, ecx: u32, edx: u32) -> u32 {
    match eax {
        SYS_DEBUG => do_sys_debug(ebx as u8),
        SYS_EXIT => do_sys_exit((ebx as i32) as isize),
        SYS_READ => eax = do_sys_read(ebx as usize, ecx as *mut u8, edx as usize) as u32,
        SYS_READ_TO_END => eax = do_sys_read_to_end(ebx as usize, ecx as *mut Vec<u8>) as u32,
        SYS_WRITE => eax = do_sys_write(ebx as usize, ecx as *mut u8, edx as usize) as u32,
        SYS_OPEN => eax = do_sys_open(ebx as *mut u8, (ecx as i32) as isize, (edx as i32) as isize) as u32,
        SYS_CLOSE => eax = do_sys_close(ebx as usize) as u32,
        SYS_TIME => {
            let reenable = start_no_ints();

            if ecx == 0 {
                ptr::write(ebx as *mut Duration, ::clock_monotonic);
            }else{
                ptr::write(ebx as *mut Duration, ::clock_realtime);
            }

            end_no_ints(reenable);
        },
        SYS_BRK => eax = do_sys_brk(ebx as usize) as u32,
        SYS_YIELD => context_switch(false),

        SYS_TRIGGER => {
            let mut event = ptr::read(ebx as *const Event);

            let reenable = start_no_ints();

            if event.code == 'm' {
                event.a = max(0, min((*::session_ptr).display.width as isize - 1, (*::session_ptr).mouse_point.x + event.a));
                event.b = max(0, min((*::session_ptr).display.height as isize - 1, (*::session_ptr).mouse_point.y + event.b));
                (*::session_ptr).mouse_point.x = event.a;
                (*::session_ptr).mouse_point.y = event.b;
                (*::session_ptr).redraw = max((*::session_ptr).redraw, REDRAW_CURSOR);
            }
            if event.code == 'k' && event.b == 0x3B && event.c > 0 {
                ::debug_draw = true;
                ::debug_redraw = true;
            }
            if event.code == 'k' && event.b == 0x3C && event.c > 0 {
                ::debug_draw = false;
                (*::session_ptr).redraw = max((*::session_ptr).redraw, REDRAW_ALL);
            }

            //TODO: Dispatch to appropriate window
            (*::events_ptr).push(event);

            end_no_ints(reenable);
        },
        SYS_WINDOW_CREATE => {
            let reenable = start_no_ints();

            (*::session_ptr).add_window(ebx as *mut Window);

            end_no_ints(reenable);
        },
        SYS_WINDOW_DESTROY => {
            let reenable = start_no_ints();

            (*::session_ptr).remove_window(ebx as *mut Window);

            end_no_ints(reenable);
        },
        SYS_ALLOC => eax = alloc(ebx as usize) as u32,
        SYS_REALLOC => eax = realloc(ebx as usize, ecx as usize) as u32,
        SYS_REALLOC_INPLACE => eax = realloc_inplace(ebx as usize, ecx as usize) as u32,
        SYS_UNALLOC => unalloc(ebx as usize),
        _ => {
            d("Unknown Syscall: ");
            dd(eax as usize);
            d(", ");
            dh(ebx as usize);
            d(", ");
            dh(ecx as usize);
            d(", ");
            dh(edx as usize);
            dl();
        }
    }

    return eax;
}
