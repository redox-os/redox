//! Intrinsics for panic handling

use arch::interrupt::halt;

#[cfg(not(test))]
#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

#[cfg(not(test))]
/// Required to handle panics
#[lang = "panic_fmt"]
extern "C" fn panic_fmt(fmt: ::core::fmt::Arguments, file_line: &(&'static str, u32)) -> ! {
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
