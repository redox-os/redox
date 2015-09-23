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
        SYS_EXIT => context_exit(),
        SYS_WRITE => {
            if ebx == 1 || ebx == 2 {
                let mut ptr = ecx as *const u8;
                for i in 0..edx as usize{
                    db(ptr::read(ptr.offset(i as isize)));
                }
            }else{
                d("Write: Unknown File ");
                dh(ebx as usize);
                dl();
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
