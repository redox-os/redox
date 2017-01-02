///! Syscall handlers

extern crate syscall;

pub use self::syscall::{data, error, flag, number, scheme};

pub use self::driver::*;
pub use self::fs::*;
pub use self::futex::futex;
pub use self::privilege::*;
pub use self::process::*;
pub use self::time::*;
pub use self::validate::*;

use self::data::TimeSpec;
use self::error::{Error, Result, ENOSYS};
use self::number::*;

use context::ContextId;
use scheme::{FileHandle, SchemeNamespace};

/// Driver syscalls
pub mod driver;

/// Filesystem syscalls
pub mod fs;

/// Fast userspace mutex
pub mod futex;

/// Privilege syscalls
pub mod privilege;

/// Process syscalls
pub mod process;

/// Time syscalls
pub mod time;

/// Validate input
pub mod validate;

#[no_mangle]
pub extern fn syscall(a: usize, b: usize, c: usize, d: usize, e: usize, f: usize, stack: usize) -> usize {
    #[inline(always)]
    fn inner(a: usize, b: usize, c: usize, d: usize, e: usize, f: usize, stack: usize) -> Result<usize> {
        match a & SYS_CLASS {
            SYS_CLASS_FILE => {
                let fd = FileHandle::from(b);
                match a & SYS_ARG {
                    SYS_ARG_SLICE => file_op_slice(a, fd, validate_slice(c as *const u8, d)?),
                    SYS_ARG_MSLICE => file_op_mut_slice(a, fd, validate_slice_mut(c as *mut u8, d)?),
                    _ => match a {
                        SYS_CLOSE => close(fd),
                        SYS_DUP => dup(fd, validate_slice(c as *const u8, d)?).map(FileHandle::into),
                        SYS_DUP2 => dup2(fd, FileHandle::from(c), validate_slice(d as *const u8, e)?).map(FileHandle::into),
                        SYS_FEVENT => fevent(fd, c),
                        SYS_FUNMAP => funmap(b),
                        SYS_DUP_EXPORT => dup_export(fd, ContextId::from(c), validate_slice(d as *const u8, e)?),
                        _ => file_op(a, fd, c, d)
                    }
                }
            },
            SYS_CLASS_PATH => match a {
                SYS_OPEN => open(validate_slice(b as *const u8, c)?, d).map(FileHandle::into),
                SYS_CHMOD => chmod(validate_slice(b as *const u8, c)?, d as u16),
                SYS_RMDIR => rmdir(validate_slice(b as *const u8, c)?),
                SYS_UNLINK => unlink(validate_slice(b as *const u8, c)?),
                _ => unreachable!()
            },
            _ => match a {
                SYS_DUP_FROM => dup_import(ContextId::from(b), validate_slice(c as *const u8, d)?).map(FileHandle::into),
                SYS_YIELD => sched_yield(),
                SYS_NANOSLEEP => nanosleep(validate_slice(b as *const TimeSpec, 1).map(|req| &req[0])?, validate_slice_mut(c as *mut TimeSpec, 1).ok().map(|rem| &mut rem[0])),
                SYS_CLOCK_GETTIME => clock_gettime(b, validate_slice_mut(c as *mut TimeSpec, 1).map(|time| &mut time[0])?),
                SYS_FUTEX => futex(validate_slice_mut(b as *mut i32, 1).map(|uaddr| &mut uaddr[0])?, c, d as i32, e, f as *mut i32),
                SYS_BRK => brk(b),
                SYS_GETPID => getpid().map(ContextId::into),
                SYS_CLONE => clone(b, stack).map(ContextId::into),
                SYS_EXIT => exit((b & 0xFF) << 8),
                SYS_KILL => kill(ContextId::from(b), c),
                SYS_WAITPID => waitpid(ContextId::from(b), c, d).map(ContextId::into),
                SYS_CHDIR => chdir(validate_slice(b as *const u8, c)?),
                SYS_EXECVE => exec(validate_slice(b as *const u8, c)?, validate_slice(d as *const [usize; 2], e)?),
                SYS_IOPL => iopl(b, stack),
                SYS_GETCWD => getcwd(validate_slice_mut(b as *mut u8, c)?),
                SYS_GETEGID => getegid(),
                SYS_GETENS => getens(),
                SYS_GETEUID => geteuid(),
                SYS_GETGID => getgid(),
                SYS_GETNS => getns(),
                SYS_GETUID => getuid(),
                SYS_MKNS => mkns(validate_slice(b as *const [usize; 2], c)?),
                SYS_SETREUID => setreuid(b as u32, c as u32),
                SYS_SETRENS => setrens(SchemeNamespace::from(b), SchemeNamespace::from(c)),
                SYS_SETREGID => setregid(b as u32, c as u32),
                SYS_PIPE2 => pipe2(validate_slice_mut(b as *mut usize, 2)?, c),
                SYS_PHYSALLOC => physalloc(b),
                SYS_PHYSFREE => physfree(b, c),
                SYS_PHYSMAP => physmap(b, c, d),
                SYS_PHYSUNMAP => physunmap(b),
                SYS_VIRTTOPHYS => virttophys(b),
                _ => Err(Error::new(ENOSYS))
            }
        }
    }

    let result = inner(a, b, c, d, e, f, stack);

    /*
    if let Err(ref err) = result {
        let contexts = ::context::contexts();
        if let Some(context_lock) = contexts.current() {
            let context = context_lock.read();
            print!("{}: {}: ", unsafe { ::core::str::from_utf8_unchecked(&context.name.lock()) }, context.id.into());
        }

        println!("{:X}, {:X}, {:X}, {:X}: {}", a, b, c, d, err);
    }
    */

    Error::mux(result)
}
