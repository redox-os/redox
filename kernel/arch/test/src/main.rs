/// This function is where the kernel sets up IRQ handlers
/// It is increcibly unsafe, and should be minimal in nature

extern {
    fn kmain() -> !;
}

#[no_mangle]
pub unsafe extern fn kstart() -> ! {
    kmain();
}
