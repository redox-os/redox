#![feature(asm)]
#![feature(thread_local)]

use std::thread;

#[thread_local]
static mut TLS_DATA: usize = 1;
#[thread_local]
static mut TLS_BSS: usize = 0;

fn main() {
    unsafe { asm!("xchg bx, bx" : : : "memory" : "intel", "volatile") };

    unsafe {
        TLS_DATA += 1;
        TLS_BSS += 1;
        println!("PARENT: DATA {}==2 BSS {}==1", TLS_DATA, TLS_BSS);
    }

    thread::spawn(|| {
        unsafe {
            TLS_DATA += 1;
            TLS_BSS += 1;
            println!("CHILD:  DATA {}==2 BSS {}==1", TLS_DATA, TLS_BSS);
        }
    }).join().unwrap();
}
