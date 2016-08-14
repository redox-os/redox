///! Process syscalls

use arch::interrupt::halt;

use super::{convert_slice, Result};

pub fn exit(status: usize) -> ! {
    println!("Exit {}", status);
    loop {
        unsafe { halt() };
    }
}

pub fn exec(path: &[u8], args: &[[usize; 2]]) -> Result<usize> {
    print!("Exec {:?}", ::core::str::from_utf8(path));
    for arg in args {
        print!(" {:?}", ::core::str::from_utf8(convert_slice(arg[0] as *const u8, arg[1])?));
    }
    println!("");
    Ok(0)
}
