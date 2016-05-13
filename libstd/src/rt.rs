use core::{fmt, mem, ptr, slice, str};
use panic::panic_impl;
use env::{args_init, args_destroy};
use system::syscall::sys_exit;
use vec::Vec;

pub fn begin_panic(string: &'static str, file_line: &(&'static str, u32)) -> ! {
    let &(file, line) = file_line;
    panic_impl(&format_args!("{}", string), file, line)
}

pub fn begin_panic_fmt(fmt: &fmt::Arguments, file_line: &(&'static str, u32)) -> ! {
    let &(file, line) = file_line;
    panic_impl(fmt, file, line)
}

pub fn begin_unwind(string: &'static str, file_line: &(&'static str, u32)) -> ! {
    let &(file, line) = file_line;
    panic_impl(&format_args!("{}", string), file, line)
}

pub fn begin_unwind_fmt(fmt: &fmt::Arguments, file_line: &(&'static str, u32)) -> ! {
    let &(file, line) = file_line;
    panic_impl(fmt, file, line)
}

#[no_mangle]
#[naked]
#[cfg(target_arch = "x86")]
pub unsafe fn _start() {
    asm!("push esp
        call _start_stack
        pop esp"
        :
        :
        : "memory"
        : "intel", "volatile");
    let _ = sys_exit(0);
}

#[no_mangle]
#[naked]
#[cfg(target_arch = "x86_64")]
pub unsafe fn _start() {
    asm!("push rsp
        call _start_stack
        pop rsp"
        :
        :
        : "memory"
        : "intel", "volatile");
    let _ = sys_exit(0);
}

#[no_mangle]
pub unsafe extern "C" fn _start_stack(stack: *const usize){
    extern "C" {
        fn main(argc: usize, argv: *const *const u8) -> usize;
    }

    //asm!("xchg bx, bx" : : : "memory" : "intel", "volatile");

    let argc = *stack;
    let argv = stack.offset(1) as *const *const u8;
    let _ = sys_exit(main(argc, argv));
}

#[lang = "start"]
fn lang_start(main: *const u8, argc: usize, argv: *const *const u8) -> usize {
    unsafe {
        let mut args: Vec<&'static str> = Vec::new();
        for i in 0..argc as isize {
            let arg = ptr::read(argv.offset(i));
            if arg as usize > 0 {
                let mut len = 0;
                for j in 0..4096 {
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

        mem::transmute::<_, fn()>(main)();

        args_destroy();
    }

    0
}
