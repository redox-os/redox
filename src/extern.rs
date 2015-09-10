pub fn unsupported(){
    unsafe{ asm!("int 3" : : : : "intel", "volatile") }
}

#[allow(unused_variables)]
#[no_mangle]
pub extern fn fmod(x: f64, y: f64) -> f64 {
    unsupported();
    return 0.0;
}

#[allow(unused_variables)]
#[no_mangle]
pub extern fn fmodf(x: f32, y: f32) -> f32 {
    unsupported();
    return 0.0;
}

#[allow(unused_variables)]
#[no_mangle]
pub extern fn __powisf2(a: f32, x: i32) -> f32 {
    unsupported();
    return 0.0;
}

#[allow(unused_variables)]
#[no_mangle]
pub extern fn __powidf2(a: f64, x: i32) -> f64 {
    unsupported();
    return 0.0;
}

#[no_mangle]
pub extern fn __mulodi4(a: i32, b: i32, overflow: *mut i32) -> i32 {
    let result = (a as i64) * (b as i64);
    if result > 2 << 32 {
        unsafe{
            ptr::write(overflow, 1);
        }
    }
    return result as i32;
}

#[no_mangle]
pub extern fn __moddi3(a: i32, b: i32) -> i32 {
    return a%b;
}

#[no_mangle]
pub extern fn __divdi3(a: i32, b: i32) -> i32 {
    return a/b;
}

#[no_mangle]
pub extern fn __umoddi3(a: u32, b: u32) -> u32 {
    return a%b;
}

#[no_mangle]
pub extern fn __udivdi3(a: u32, b: u32) -> u32 {
    return a/b;
}
