use core::cmp::max;
use core::cmp::min;
use core::ptr;
use core::slice;

use common::context::*;
use common::debug::*;
use common::event::*;
use common::memory::*;
use common::resource::*;
use common::scheduler::*;
use common::string::*;
use common::time::*;

use drivers::pio::*;

use graphics::color::*;
use graphics::size::*;
use graphics::window::*;

use syscall::common::*;

pub unsafe fn do_sys_debug(byte: u8) {
    let reenable = start_no_ints();

    if ::debug_display as usize > 0 {
        let display = &*(*::debug_display);
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
        display.rect(::debug_point, Size::new(8, 16), Color::new(255, 255, 255));
        ::debug_redraw = true;
        //If interrupts disabled, probably booting up
        if !reenable && ::debug_draw && ::debug_redraw {
            ::debug_redraw = false;
            display.flip();
        }
    }

    loop {
        if inb(0x3F8 + 5) & 0x20 == 0x20 {
            break;
        }
    }
    outb(0x3F8, byte);

    end_no_ints(reenable);
}

pub unsafe fn do_sys_exit(status: isize) {
    context_exit();
}

pub unsafe fn do_sys_read(fd: usize, buf: *mut u8, count: usize) -> usize {
    let mut ret = 0xFFFFFFFF;

    let reenable = start_no_ints();

    let contexts = & *contexts_ptr;
    if let Option::Some(current) = contexts.get(context_i) {
        for file in current.files.iter() {
            if file.fd == fd {
                end_no_ints(reenable);

                if let Option::Some(count) = file.resource.read(slice::from_raw_parts_mut(buf, count)) {
                    ret = count;
                }

                start_no_ints();

                break;
            }
        }
    }

    end_no_ints(reenable);

    ret
}

pub unsafe fn do_sys_write(fd: usize, buf: *const u8, count: usize) -> usize {
    let mut ret = 0xFFFFFFFF;

    let reenable = start_no_ints();

    let contexts = & *contexts_ptr;
    if let Option::Some(current) = contexts.get(context_i) {
        for file in current.files.iter() {
            if file.fd == fd {
                end_no_ints(reenable);

                if let Option::Some(count) = file.resource.write(slice::from_raw_parts(buf, count)) {
                    ret = count;
                }

                start_no_ints();

                break;
            }
        }
    }

    end_no_ints(reenable);

    ret
}

pub unsafe fn do_sys_open(path: *const u8, flags: isize, mode: isize) -> usize {
    let mut path_str = String::from_c_str(path);

    //TODO: Handle more path derivatives

    if path_str.find(":".to_string()).is_none() {
        let reenable = start_no_ints();

        let contexts = & *contexts_ptr;
        if let Option::Some(current) = contexts.get(context_i) {
            path_str = current.cwd.clone() + path_str;
        }

        end_no_ints(reenable);
    }

    let resource = (*::session_ptr).open(&URL::from_string(&path_str));

    let mut fd = 0xFFFFFFFF;

    let reenable = start_no_ints();

    let contexts = & *contexts_ptr;
    if let Option::Some(mut current) = contexts.get(context_i) {
        fd = 0;
        for file in current.files.iter() {
            if file.fd >= fd {
                fd = file.fd + 1;
            }
        }

        current.files.push(ContextFile {
            fd: fd,
            resource: resource
        });
    }

    end_no_ints(reenable);

    fd
}

pub unsafe fn do_sys_close(fd: usize) -> usize {
    let mut ret = 0xFFFFFFFF;

    let reenable = start_no_ints();

    let contexts = & *contexts_ptr;
    if let Option::Some(mut current) = contexts.get(context_i) {
        for i in 0..current.files.len() {
            let mut remove = false;
            if let Option::Some(file) = current.files.get(i) {
                if file.fd == fd {
                    remove = true;
                }
            }

            if remove {
                if let Option::Some(file) = current.files.remove(i) {
                    end_no_ints(reenable);

                    drop(file);

                    start_no_ints();

                    ret = 0;
                }

                break;
            }
        }
    }

    end_no_ints(reenable);

    ret
}

pub unsafe fn do_sys_fsync(fd: usize) -> usize {
    let mut ret = 0xFFFFFFFF;

    let reenable = start_no_ints();

    let contexts = & *contexts_ptr;
    if let Option::Some(mut current) = contexts.get(context_i) {
        for i in 0..current.files.len() {
            let mut remove = false;
            if let Option::Some(file) = current.files.get(i) {
                if file.fd == fd {
                    end_no_ints(reenable);

                    file.resource.flush();

                    start_no_ints();

                    ret = 0;
                    break;
                }
            }
        }
    }

    end_no_ints(reenable);

    ret
}

pub unsafe fn do_sys_lseek(fd: usize, offset: isize, whence: usize) -> usize {
    let mut ret = 0xFFFFFFFF;

    let reenable = start_no_ints();

    let contexts = & *contexts_ptr;
    if let Option::Some(current) = contexts.get(context_i) {
        for file in current.files.iter() {
            if file.fd == fd {
                end_no_ints(reenable);

                match whence {
                    0 => if let Option::Some(count) = file.resource.seek(ResourceSeek::Start(offset as usize)) {
                        ret = count;
                    },
                    1 => if let Option::Some(count) = file.resource.seek(ResourceSeek::Current(offset)) {
                        ret = count;
                    },
                    2 => if let Option::Some(count) = file.resource.seek(ResourceSeek::End(offset)) {
                        ret = count;
                    },
                    _ => ()
                }

                start_no_ints();

                break;
            }
        }
    }

    end_no_ints(reenable);

    ret
}

pub unsafe fn do_sys_gettimeofday(tv: *mut usize, tz: *mut isize) -> usize {
    let reenable = start_no_ints();

    if tv as usize > 0 {
        ptr::write(tv.offset(0), ::clock_realtime.secs as usize);
        ptr::write(tv.offset(1), (::clock_realtime.nanos/1000) as usize);
    }
    if tz as usize > 0 {
        ptr::write(tz.offset(0), 0);
        ptr::write(tz.offset(1), 0);
    }

    end_no_ints(reenable);

    0
}

#[inline(never)]
pub unsafe fn do_sys_brk(addr: usize) -> usize {
    let mut ret = 0;

    let reenable = start_no_ints();

    let contexts = & *contexts_ptr;
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
                    } else {
                        d("BRK: Realloc Failed\n");
                    }
                } else {
                    d("BRK: Address not in correct space\n");
                }
            } else {
                d("BRK: Memory not found\n");
            }

            current.map();
        } else {
            d("BRK: Context not found\n");
        }
    } else {
        d("BRK: Contexts disabled\n");
    }

    end_no_ints(reenable);

    ret
}

#[inline(never)]
pub unsafe fn syscall_handle(mut eax: u32, ebx: u32, ecx: u32, edx: u32) -> u32 {
    match eax {
        SYS_DEBUG => do_sys_debug(ebx as u8),

        //Linux
        SYS_EXIT => do_sys_exit((ebx as i32) as isize),
        SYS_READ => eax = do_sys_read(ebx as usize, ecx as *mut u8, edx as usize) as u32,
        SYS_WRITE => eax = do_sys_write(ebx as usize, ecx as *mut u8, edx as usize) as u32,
        SYS_OPEN => eax = do_sys_open(ebx as *mut u8, (ecx as i32) as isize, (edx as i32) as isize) as u32,
        SYS_CLOSE => eax = do_sys_close(ebx as usize) as u32,
        SYS_FSYNC => eax = do_sys_fsync(ebx as usize) as u32,
        SYS_LSEEK => eax = do_sys_lseek(ebx as usize, (ecx as i32) as isize, edx as usize) as u32,
        SYS_BRK => eax = do_sys_brk(ebx as usize) as u32,
        SYS_GETTIMEOFDAY => eax = do_sys_gettimeofday(ebx as *mut usize, ecx as *mut isize) as u32,
        SYS_YIELD => context_switch(false),

        //Rust Memory
        SYS_ALLOC => eax = alloc(ebx as usize) as u32,
        SYS_REALLOC => eax = realloc(ebx as usize, ecx as usize) as u32,
        SYS_REALLOC_INPLACE => eax = realloc_inplace(ebx as usize, ecx as usize) as u32,
        SYS_UNALLOC => unalloc(ebx as usize),

        //Windows
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

        //Misc
        SYS_TIME => {
            let reenable = start_no_ints();

            if ecx == 0 {
                ptr::write(ebx as *mut Duration, ::clock_monotonic);
            } else {
                ptr::write(ebx as *mut Duration, ::clock_realtime);
            }

            end_no_ints(reenable);
        },
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

    eax
}
