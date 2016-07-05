extern crate system;

use system::syscall::sys_iopl;

use power::reset;

mod power;

fn main() {
    unsafe { sys_iopl(3).unwrap() };

    println!("Performing reset");

    reset();
}
