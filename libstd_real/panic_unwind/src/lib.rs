#![no_std]
#![feature(core_intrinsics)]
#![feature(lang_items)]
#![feature(panic_runtime)]
#![panic_runtime]

#[no_mangle]
pub unsafe extern fn __rust_maybe_catch_panic(f: fn(*mut u8), data: *mut u8,
                                              _data_ptr: *mut usize, _vtable_ptr: *mut usize) -> u32 {
    f(data);
    0
}

#[no_mangle]
pub unsafe extern fn __rust_start_panic(_data: usize, _vtable: usize) -> u32 {
    core::intrinsics::abort();
}

#[lang = "eh_personality"]
pub extern fn eh_personality() {}

#[allow(non_snake_case)]
#[no_mangle]
/// Required to handle panics
pub unsafe extern "C" fn _Unwind_Resume() -> ! {
    core::intrinsics::abort();
}
