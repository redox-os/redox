pub use system::error::*;
pub use system::syscall::*;

use arch::regs::Regs;
use arch::context::context_switch;

pub mod execute;
pub mod fs;
pub mod memory;
pub mod process;
pub mod time;

pub fn name(number: usize) -> &'static str {
    match number {
        // Redox
        SYS_SUPERVISE => "supervise",

        // Unix
        SYS_BRK => "brk",
        SYS_CHDIR => "chdir",
        SYS_CLONE => "clone",
        SYS_CLOSE => "close",
        SYS_CLOCK_GETTIME => "clock_gettime",
        SYS_DUP => "dup",
        SYS_EXECVE => "execve",
        SYS_EXIT => "exit",
        SYS_FPATH => "fpath",
        SYS_FSTAT => "fstat",
        SYS_FSYNC => "fsync",
        SYS_FTRUNCATE => "ftruncate",
        SYS_FUTEX => "futex",
        SYS_GETPID => "getpid",
        SYS_IOPL => "iopl",
        // TODO: link
        SYS_LSEEK => "lseek",
        SYS_MKDIR => "mkdir",
        SYS_NANOSLEEP => "nanosleep",
        SYS_OPEN => "open",
        SYS_PIPE2 => "pipe2",
        SYS_READ => "read",
        SYS_RMDIR => "rmdir",
        SYS_UNLINK => "unlink",
        SYS_WAITPID => "waitpid",
        SYS_WRITE => "write",
        SYS_YIELD => "yield",

        _ => "unknown",
    }
}

/// Handle the syscall defined by the given registers.
///
/// AX defines which syscall to use. The arguments are provided in other registers, as specified by
/// the specific sycall.
///
/// The return value is placed in AX, unless otherwise specified.
pub fn handle(regs: &mut Regs) {
    let contexts = unsafe { &mut *::env().contexts.get() };
    let cur = contexts.current_mut().unwrap();
    cur.current_syscall = Some((regs.ip, regs.ax, regs.bx, regs.cx, regs.dx));
    // debugln!("PID {}: {} @ {:X}: {} {} {:X} {:X} {:X}", cur.pid, cur.name, regs.ip, regs.ax, name(regs.ax), regs.bx, regs.cx, regs.dx);
    if cur.supervised {
        // Block the process.
        cur.blocked_syscall = true;
        cur.block("syscall::handle Supervise");
        // Clear the timer.
        cur.wake = None;

        loop {
            if cur.blocked > 0 {
                unsafe { context_switch() };
            } else {
                return;
            }
        }
    }

    macro_rules! check {
        ( $r:expr ) => (
            match $r {
                Err(result) => {
                    regs.ax = -result.errno as usize;
                    return;
                },
                Ok(val) => val,
            }
        )
    }

    macro_rules! get_ref {
        ( $buf:ident, $typ:ty ) => ( check!(cur.get_ref(regs.$buf as *const $typ)) );
    }

    macro_rules! get_ref_mut {
        ( $buf:ident, $typ:ty ) => ( check!(cur.get_ref_mut(regs.$buf as *mut $typ)) );
    }

    macro_rules! get_ref_mut_opt {
        ( $buf:ident, $typ:ty ) => (
            if regs.$buf != 0 {
                Some(check!(cur.get_ref_mut(regs.$buf as *mut $typ)))
            } else {
                None
            }
        );
    }

    macro_rules! get_slice {
        ( $buf:ident, $len:ident ) => ( check!(cur.get_slice(regs.$buf as *const u8, regs.$len)) );
    }

    macro_rules! get_slice_mut {
        ( $buf:ident, $len:ident ) => ( check!(cur.get_slice_mut(regs.$buf as *mut u8, regs.$len)) );
    }

    let result = match regs.ax {
        // These are arranged in such a way that the most frequent syscalls preceeds less frequent
        // once, to acheive the best performance.

        SYS_YIELD => process::sched_yield(),
        SYS_FUTEX => process::futex(get_ref_mut!(bx, i32), regs.cx, (regs.dx as isize) as i32, regs.si, regs.di as *mut i32),
        SYS_WRITE => fs::write(regs.bx, get_slice!(cx, dx)),
        SYS_READ => fs::read(regs.bx, get_slice_mut!(cx, dx)),
        SYS_LSEEK => fs::lseek(regs.bx, regs.cx as isize, regs.dx),
        SYS_OPEN => fs::open(get_slice!(bx, cx), regs.dx),
        SYS_CLOSE => fs::close(regs.bx),
        SYS_CLONE => process::clone(regs),
        SYS_MKDIR => fs::mkdir(get_slice!(bx, cx), regs.dx),
        SYS_NANOSLEEP => time::nanosleep(get_ref!(bx, TimeSpec), get_ref_mut_opt!(cx, TimeSpec)),
        SYS_FPATH => fs::fpath(regs.bx, get_slice_mut!(cx, dx)),
        SYS_FSTAT => fs::fstat(regs.bx, get_ref_mut!(cx, Stat)),
        SYS_FSYNC => fs::fsync(regs.bx),
        SYS_FTRUNCATE => fs::ftruncate(regs.bx, regs.cx),
        SYS_DUP => fs::dup(regs.bx),
        SYS_IOPL => process::iopl(regs),
        SYS_CLOCK_GETTIME => time::clock_gettime(regs.bx, get_ref_mut!(cx, TimeSpec)),
        SYS_EXECVE => process::execve(regs.bx as *const u8, regs.cx as *const *const u8),
        SYS_EXIT => process::exit(regs.bx),
        SYS_GETPID => process::getpid(),
        // TODO: link
        SYS_PIPE2 => fs::pipe2(get_ref_mut!(bx, [usize; 2]), regs.cx),
        SYS_RMDIR => fs::rmdir(get_slice!(bx, cx)),
        SYS_UNLINK => fs::unlink(get_slice!(bx, cx)),
        SYS_WAITPID => process::waitpid(regs.bx as isize, get_ref_mut_opt!(cx, usize), regs.dx),
        SYS_BRK => memory::brk(regs.bx),
        SYS_CHDIR => fs::chdir(get_slice!(bx, cx)),
        SYS_SUPERVISE => process::supervise(regs.bx),
        _ => Err(Error::new(ENOSYS)),
    };

    {
        let contexts = unsafe { &mut *::env().contexts.get() };
        if let Ok(cur) = contexts.current_mut() {
            // debugln!("PID {}: {} @ {:X}: {} {} {:X} {:X} {:X} = {:?}", cur.pid, cur.name, regs.ip, regs.ax, name(regs.ax), regs.bx, regs.cx, regs.dx, result);
            cur.current_syscall = None;
        }
    }

    regs.ax = Error::mux(result);
}
