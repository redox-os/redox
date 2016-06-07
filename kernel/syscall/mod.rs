pub use system::error::*;
pub use system::syscall::*;

pub use self::file::*;
pub use self::memory::*;
pub use self::process::*;
pub use self::time::*;

use arch::regs::Regs;
use arch::context::context_switch;

pub mod execute;
pub mod file;
pub mod memory;
pub mod process;
pub mod time;

pub fn syscall_name(number: usize) -> &'static str {
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
        SYS_STAT => "stat",
        SYS_UNLINK => "unlink",
        SYS_WAITPID => "waitpid",
        SYS_WRITE => "write",
        SYS_YIELD => "yield",

        _ => "unknown",
    }
}

pub fn syscall_handle(regs: &mut Regs) {
    {
        let mut contexts = ::env().contexts.lock();
        if let Ok(cur) = contexts.current_mut() {
            cur.current_syscall = Some((regs.ip, regs.ax, regs.bx, regs.cx, regs.dx));
            //serial_log(format!("PID {}: {} @ {:X}: {} {} {:X} {:X} {:X}\n", cur.pid, cur.name, regs.ip, regs.ax, syscall_name(regs.ax), regs.bx, regs.cx, regs.dx).as_bytes());
            if cur.supervised {
                // Block the process.
                cur.blocked_syscall = true;
                cur.blocked = true;
                // Clear the timer.
                cur.wake = None;

                loop {
                    if cur.blocked {
                        unsafe { context_switch() };
                    } else {
                        return;
                    }
                }
            }
        }
    }

    let result = match regs.ax {
        // Redox
        SYS_SUPERVISE => do_sys_supervise(regs.bx),

        // Unix
        SYS_BRK => do_sys_brk(regs.bx),
        SYS_CHDIR => do_sys_chdir(regs.bx as *const u8),
        SYS_CLONE => do_sys_clone(regs),
        SYS_CLOSE => do_sys_close(regs.bx),
        SYS_CLOCK_GETTIME => do_sys_clock_gettime(regs.bx, regs.cx as *mut TimeSpec),
        SYS_DUP => do_sys_dup(regs.bx),
        SYS_EXECVE => do_sys_execve(regs.bx as *const u8, regs.cx as *const *const u8),
        SYS_EXIT => do_sys_exit(regs.bx),
        SYS_FPATH => do_sys_fpath(regs.bx, regs.cx as *mut u8, regs.dx),
        SYS_FSTAT => do_sys_fstat(regs.bx, regs.cx as *mut Stat),
        SYS_FSYNC => do_sys_fsync(regs.bx),
        SYS_FTRUNCATE => do_sys_ftruncate(regs.bx, regs.cx),
        SYS_GETPID => do_sys_getpid(),
        SYS_IOPL => do_sys_iopl(regs),
        // TODO: link
        SYS_LSEEK => do_sys_lseek(regs.bx, regs.cx as isize, regs.dx),
        SYS_MKDIR => do_sys_mkdir(regs.bx as *const u8, regs.cx),
        SYS_NANOSLEEP => do_sys_nanosleep(regs.bx as *const TimeSpec, regs.cx as *mut TimeSpec),
        SYS_OPEN => do_sys_open(regs.bx as *const u8, regs.cx),
        SYS_PIPE2 => do_sys_pipe2(regs.bx as *mut usize, regs.cx),
        SYS_READ => do_sys_read(regs.bx, regs.cx as *mut u8, regs.dx),
        SYS_RMDIR => do_sys_rmdir(regs.bx as *const u8),
        SYS_STAT => do_sys_stat(regs.bx as *const u8, regs.cx as *mut Stat),
        SYS_UNLINK => do_sys_unlink(regs.bx as *const u8),
        SYS_WAITPID => do_sys_waitpid(regs.bx as isize, regs.cx as *mut usize, regs.dx),
        SYS_WRITE => do_sys_write(regs.bx, regs.cx as *mut u8, regs.dx),
        SYS_YIELD => do_sys_yield(),

        _ => Err(Error::new(ENOSYS)),
    };

    {
        let mut contexts = ::env().contexts.lock();
        if let Ok(cur) = contexts.current_mut() {
            //serial_log(format!("PID {}: {} @ {:X}: {} {} {:X} {:X} {:X} = {:?}\n", cur.pid, cur.name, regs.ip, regs.ax, syscall_name(regs.ax), regs.bx, regs.cx, regs.dx, result).as_bytes());
            cur.current_syscall = None;
        }
    }

    regs.ax = Error::mux(result);
}
