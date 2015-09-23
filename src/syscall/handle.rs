use alloc::boxed::*;

use core::cmp::max;
use core::cmp::min;
use core::ptr;

use common::context::*;
use common::event::*;
use common::pio::*;
use common::resource::*;
use common::scheduler::*;
use common::time::*;

use graphics::color::*;
use graphics::window::*;

use syscall::common::*;

pub unsafe fn syscall_handle(eax: u32, ebx: u32, ecx: u32, edx: u32){
    match eax {
        SYS_DEBUG => { //Debug
            let reenable = start_no_ints();

            if ::debug_display as usize > 0 {
                let display = &*(*::debug_display);
                if ebx == 10 {
                    ::debug_point.x = 0;
                    ::debug_point.y += 16;
                    ::debug_redraw = true;
                }else{
                    display.char(::debug_point, (ebx as u8) as char, Color::new(255, 255, 255));
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

            outb(0x3F8, ebx as u8);

            end_no_ints(reenable);
        },
        SYS_EXIT => context_exit(),
        SYS_OPEN => {
            //Not interrupt-locked to avoid slowness

            let session = &mut *::session_ptr;
            let url = &*(ebx as *const URL);

            ptr::write(ecx as *mut Box<Resource>, session.open(url));
        },
        SYS_TIME => {
            let reenable = start_no_ints();

            if ecx == 0 {
                ptr::write(ebx as *mut Duration, ::clock_monotonic);
            }else{
                ptr::write(ebx as *mut Duration, ::clock_realtime);
            }

            end_no_ints(reenable);
        },
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
        SYS_YIELD => context_switch(false),
        _ => ()
    }
}
