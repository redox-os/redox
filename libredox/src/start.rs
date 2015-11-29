use super::{ptr, slice, str};
use super::env::{args_init, args_destroy};
use super::syscall::sys_exit;
use super::Vec;

extern {
    fn main();
}

#[no_mangle]
#[inline(never)]
pub unsafe extern fn _start_stack(stack: *const usize) {
    let mut args: Vec<&'static str> = Vec::new();
    //TODO: Fix issue with stack not being in context VM space
    let argc = ptr::read(stack);
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
