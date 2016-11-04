///! Syscall handlers

extern crate syscall;

pub use self::syscall::{data, error, flag, number, scheme};

pub use self::fs::*;
pub use self::futex::futex;
pub use self::process::*;
pub use self::time::*;
pub use self::validate::*;

use self::data::TimeSpec;
use self::error::{Error, Result, ENOSYS};
use self::number::*;

/// Filesystem syscalls
pub mod fs;

/// Fast userspace mutex
pub mod futex;

/// Process syscalls
pub mod process;

/// Time syscalls
pub mod time;

/// Validate input
pub mod validate;

/// Execute a syscall.
///
/// In `syscall(sys_id, b, c, d, e, f, stack)`,
///
/// - `sys_id` identifies the system call (see `syscall::number::SYS_*`);
/// - `b`, `c`, `d`, `d`, `f` are the *untyped* arguments of the syscall
#[no_mangle]
pub extern fn syscall(a: usize, b: usize, c: usize, d: usize, e: usize, f: usize, stack: usize) -> usize {
    #[inline(always)]
    fn inner(a: usize, b: usize, c: usize, d: usize, e: usize, f: usize, stack: usize) -> Result<usize> {
        match a & SYS_CLASS {
            SYS_CLASS_FILE => match a & SYS_ARG {
                SYS_ARG_SLICE => file_op_slice(a, b, validate_slice(c as *const u8, d)?),
                SYS_ARG_MSLICE => file_op_mut_slice(a, b, validate_slice_mut(c as *mut u8, d)?),
                _ => match a {
                    SYS_CLOSE => close(b),
                    SYS_DUP => dup(b, validate_slice(c as *const u8, d)?),
                    SYS_FEVENT => fevent(b, c),
                    _ => file_op(a, b, c, d)
                }
            },
            SYS_CLASS_PATH => match a {
                SYS_OPEN => open(validate_slice(b as *const u8, c)?, d),
                SYS_MKDIR => mkdir(validate_slice(b as *const u8, c)?, d as u16),
                SYS_RMDIR => rmdir(validate_slice(b as *const u8, c)?),
                SYS_UNLINK => unlink(validate_slice(b as *const u8, c)?),
                _ => unreachable!()
            },
            _ => match a {
                SYS_EXIT => exit(b),
                SYS_WAITPID => waitpid(b, c, d),
                SYS_EXECVE => exec(validate_slice(b as *const u8, c)?, validate_slice(d as *const [usize; 2], e)?),
                SYS_CHDIR => chdir(validate_slice(b as *const u8, c)?),
                SYS_GETPID => getpid(),
                SYS_BRK => brk(b),
                SYS_IOPL => iopl(b),
                SYS_CLONE => clone(b, stack),
                SYS_YIELD => sched_yield(),
                SYS_NANOSLEEP => nanosleep(validate_slice(b as *const TimeSpec, 1).map(|req| &req[0])?, validate_slice_mut(c as *mut TimeSpec, 1).ok().map(|rem| &mut rem[0])),
                SYS_GETCWD => getcwd(validate_slice_mut(b as *mut u8, c)?),
                SYS_GETUID => getuid(),
                SYS_GETGID => getgid(),
                SYS_GETEUID => geteuid(),
                SYS_GETEGID => getegid(),
                SYS_SETUID => setuid(b as u32),
                SYS_SETGID => setgid(b as u32),
                SYS_CLOCK_GETTIME => clock_gettime(b, validate_slice_mut(c as *mut TimeSpec, 1).map(|time| &mut time[0])?),
                SYS_FUTEX => futex(validate_slice_mut(b as *mut i32, 1).map(|uaddr| &mut uaddr[0])?, c, d as i32, e, f as *mut i32),
                SYS_PIPE2 => pipe2(validate_slice_mut(b as *mut usize, 2)?, c),
                SYS_PHYSALLOC => physalloc(b),
                SYS_PHYSFREE => physfree(b, c),
                SYS_PHYSMAP => physmap(b, c, d),
                SYS_PHYSUNMAP => physunmap(b),
                SYS_VIRTTOPHYS => virttophys(b),
                SYS_CAP_OPEN => cap_open(validate_slice(b as *const u8, c), validate_slice(d as *const u8, e), f),
                SYS_CAP_DROP => cap_drop(b, c),
                SYS_CAP_SEND => cap_send(b, c, d),
                SYS_CAP_CHECK => cap_check(b, c as *mut *mut usize, d as *mut usize),
                _ => Err(Error::new(ENOSYS))
            }
        }
    }

    let result = inner(a, b, c, d, e, f, stack);
/*
    if let Err(ref err) = result {
        println!("{}, {}, {}, {}: {}", a, b, c, d, err);
    }
*/
    Error::mux(result)
}
