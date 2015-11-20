use redox::{String, Vec};
use redox::fs::File;
use redox::io::Read;
use redox::syscall::sys_close;

pub fn main() {
    unsafe {
        sys_close(2);
        sys_close(1);
        sys_close(0);
    }

    let mut stdin = File::open("terminal:Terminal").unwrap();
    let mut stdout = stdin.dup().unwrap();
    let mut stderr = stdout.dup().unwrap();

    File::exec("file:/apps/shell/main.bin", &[]);
}
