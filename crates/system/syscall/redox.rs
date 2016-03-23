use syscall::arch::syscall2;
use error::Result;

pub const SYS_DEBUG: usize = 0;

pub fn sys_debug(buf: &[u8]) -> Result<usize> {
    unsafe { syscall2(SYS_DEBUG, buf.as_ptr() as usize, buf.len()) }
}
