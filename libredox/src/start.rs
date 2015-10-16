use ptr;
use slice;
use str;

use env::*;
use vec::Vec;

use syscall::sys_exit;

extern {
    fn main();
}

#[inline(never)]
unsafe fn _start_stack(stack: *const usize) {
    let argc = ptr::read(stack);
    let mut args: Vec<&'static str> = Vec::new();
    for i in 0..argc as isize {
        let arg = ptr::read(stack.offset(1 + i)) as *const u8;
        if arg as usize > 0 {
            let mut len = 0;
            for j in 0..4096 /* Max arg length */ {
                len = j;
                if ptr::read(arg.offset(j)) == 0 {
                    break;
                }
            }
            let utf8: &'static [u8] = slice::from_raw_parts(arg, len as usize);
            args.push(str::from_utf8_unchecked(utf8));
        }
    }

    args_init(args);
    main();
    args_destroy();
    sys_exit(0);
}

#[cold]
#[inline(never)]
#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "C" fn _start() {
    let stack: *const usize;
    asm!("" : "={esp}"(stack) : : "memory" : "intel", "volatile");
    _start_stack(stack);
}

#[cold]
#[inline(never)]
#[no_mangle]
#[cfg(target_arch = "x86_64")]
pub unsafe extern "C" fn _start() {
    let stack: *const usize;
    asm!("" : "={rsp}"(stack) : : "memory" : "intel", "volatile");
    _start_stack(stack);
}
