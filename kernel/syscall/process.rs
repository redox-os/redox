///! Process syscalls

use arch::interrupt::halt;

pub fn exit(status: usize) -> ! {
    println!("Exit {}", status);
    loop {
        unsafe { halt() };
    }
}
