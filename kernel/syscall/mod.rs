///! Syscall handlers

extern crate syscall;

pub use self::syscall::{error, number, scheme};

use self::error::{Error, Result, ENOSYS};
use self::number::*;
pub use self::fs::*;
pub use self::process::*;
pub use self::validate::*;

/// Filesystem syscalls
pub mod fs;

/// Process syscalls
pub mod process;

/// Validate input
pub mod validate;

#[no_mangle]
pub extern fn syscall(a: usize, b: usize, c: usize, d: usize, e: usize, f: usize, stack: usize) -> usize {
    #[inline(always)]
    fn inner(a: usize, b: usize, c: usize, d: usize, e: usize, _f: usize, stack: usize) -> Result<usize> {
        match a {
            SYS_EXIT => exit(b),
            SYS_READ => read(b, validate_slice_mut(c as *mut u8, d)?),
            SYS_WRITE => write(b, validate_slice(c as *const u8, d)?),
            SYS_OPEN => open(validate_slice(b as *const u8, c)?, d),
            SYS_CLOSE => close(b),
            SYS_WAITPID => waitpid(b, c, d),
            SYS_EXECVE => exec(validate_slice(b as *const u8, c)?, validate_slice(d as *const [usize; 2], e)?),
            SYS_CHDIR => chdir(validate_slice(b as *const u8, c)?),
            SYS_GETPID => getpid(),
            SYS_DUP => dup(b),
            SYS_BRK => brk(b),
            SYS_IOPL => iopl(b),
            SYS_FSYNC => fsync(b),
            SYS_CLONE => clone(b, stack),
            SYS_YIELD => sched_yield(),
            SYS_GETCWD => getcwd(validate_slice_mut(b as *mut u8, c)?),
            _ => {
                println!("Unknown syscall {}", a);
                Err(Error::new(ENOSYS))
            }
        }
    }

    Error::mux(inner(a, b, c, d, e, f, stack))
}
