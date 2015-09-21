use alloc::boxed::*;

use core::cmp::max;
use core::cmp::min;
use core::ptr;

use common::context::*;
use common::debug::*;
use common::event::*;
use common::pio::*;
use common::resource::*;
use common::scheduler::*;

//Going to use linux syscall definitions from http://docs.cs.up.ac.za/programming/asm/derick_tut/syscalls.html

const SYS_EXIT: u32 = 1;
const SYS_READ: u32 = 3;
const SYS_WRITE: u32 = 4;
const SYS_OPEN: u32 = 5;
const SYS_CLOSE: u32 = 6;

pub unsafe fn linux_handle(eax: u32, ebx: u32, ecx: u32, edx: u32){
    match eax {
        SYS_EXIT => {
            let reenable = start_no_ints();

            let contexts = &mut *(*contexts_ptr);

            if contexts.len() > 1 && context_i > 1 {
                let current_option = contexts.remove(context_i);

                if context_i >= contexts.len() {
                    context_i -= contexts.len();
                }
                match current_option {
                    Option::Some(mut current) => match contexts.get(context_i) {
                        Option::Some(next) => {
                            current.remap(next);
                            current.switch(next);
                        },
                        Option::None => ()
                    },
                    Option::None => ()
                }
            }

            end_no_ints(reenable);
        },
        SYS_WRITE => {
            let mut ptr = ecx as *const u8;
            loop {
                let b = ptr::read(ptr);
                if b == 0 {
                    break;
                }
                db(b);
                ptr = ptr.offset(1);
            }
        },
        _ => {
            d("Unimplemented Linux Syscall ");
            dh(eax as usize);
            d(", ");
            dh(ebx as usize);
            d(", ");
            dh(ecx as usize);
            d(", ");
            dh(edx as usize);
            dl();
        }
    }
}
