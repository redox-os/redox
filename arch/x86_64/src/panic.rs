//! Intrinsics for panic handling

use interrupt::halt;

#[cfg(not(test))]
#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

#[cfg(not(test))]
/// Required to handle panics
#[lang = "panic_fmt"]
extern "C" fn panic_fmt(fmt: ::core::fmt::Arguments, file: &str, line: u32) -> ! {
    let mut rbp: usize;
    unsafe { asm!("xchg bx, bx" : "={rbp}"(rbp) : : : "intel", "volatile"); }

    println!("PANIC: {}", fmt);
    println!("FILE: {}", file);
    println!("LINE: {}", line);

    println!("TRACE: {:>016X}", rbp);
    for i in 0..10 {
        unsafe {
            let rip = *(rbp as *const usize).offset(1);
            println!("  {:>016X}: {:>016X}", rbp, rip);
            if rip == 0 {
                break;
            }
            rbp = *(rbp as *const usize);
        }
    }

    println!("HALT");
    loop {
        unsafe { halt() };
    }
}

#[allow(non_snake_case)]
#[no_mangle]
/// Required to handle panics
pub extern "C" fn _Unwind_Resume() -> ! {
    loop {
        unsafe { halt() }
    }
}
