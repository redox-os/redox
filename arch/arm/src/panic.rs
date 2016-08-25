//! Intrinsics for panic handling

use interrupt;

#[cfg(not(test))]
#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

#[cfg(not(test))]
/// Required to handle panics
#[lang = "panic_fmt"]
extern "C" fn panic_fmt(fmt: ::core::fmt::Arguments, file: &str, line: u32) -> ! {
    println!("PANIC: {}", fmt);
    println!("FILE: {}", file);
    println!("LINE: {}", line);

    unsafe { interrupt::stack_trace(); }

    println!("HALT");
    loop {
        unsafe { interrupt::halt(); }
    }
}

#[allow(non_snake_case)]
#[no_mangle]
/// Required to handle panics
pub extern "C" fn _Unwind_Resume() -> ! {
    loop {
        unsafe { interrupt::halt(); }
    }
}

/// Required for linker
#[no_mangle]
pub extern "C" fn __aeabi_unwind_cpp_pr0() {
    loop {}
}
