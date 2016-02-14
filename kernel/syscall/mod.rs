pub use system::error::*;
pub use system::syscall::*;

pub use self::debug::*;
pub use self::file::*;
pub use self::memory::*;
pub use self::process::*;
pub use self::time::*;

use arch::regs::Regs;

pub mod debug;
pub mod execute;
pub mod file;
pub mod memory;
pub mod process;
pub mod time;

pub fn syscall_handle(regs: &mut Regs) {
    match regs.ax {
        SYS_DEBUG => do_sys_debug(regs.bx as *const u8, regs.cx),
        // Linux
        SYS_BRK => regs.ax = do_sys_brk(regs.bx),
        SYS_CHDIR => regs.ax = do_sys_chdir(regs.bx as *const u8),
        SYS_CLONE => regs.ax = do_sys_clone(regs.bx),
        SYS_CLOSE => regs.ax = do_sys_close(regs.bx),
        SYS_CLOCK_GETTIME => regs.ax = do_sys_clock_gettime(regs.bx, regs.cx as *mut TimeSpec),
        SYS_DUP => regs.ax = do_sys_dup(regs.bx),
        SYS_EXECVE => regs.ax = do_sys_execve(regs.bx as *const u8, regs.cx as *const *const u8),
        SYS_EXIT => do_sys_exit(regs.bx),
        SYS_FPATH => regs.ax = do_sys_fpath(regs.bx, regs.cx as *mut u8, regs.dx),
        // TODO: fstat
        SYS_FSYNC => regs.ax = do_sys_fsync(regs.bx),
        SYS_FTRUNCATE => regs.ax = do_sys_ftruncate(regs.bx, regs.cx),
        SYS_GETPID => regs.ax = do_sys_getpid(),
        // TODO: link
        SYS_LSEEK => regs.ax = do_sys_lseek(regs.bx, regs.cx as isize, regs.dx),
        SYS_MKDIR => regs.ax = do_sys_mkdir(regs.bx as *const u8, regs.cx),
        SYS_NANOSLEEP =>
            regs.ax = do_sys_nanosleep(regs.bx as *const TimeSpec, regs.cx as *mut TimeSpec),
        SYS_OPEN => regs.ax = do_sys_open(regs.bx as *const u8, regs.cx), //regs.cx as isize, regs.dx as isize),
        SYS_PIPE2 => regs.ax = do_sys_pipe2(regs.bx as *mut usize, regs.cx),
        SYS_READ => regs.ax = do_sys_read(regs.bx, regs.cx as *mut u8, regs.dx),
        SYS_UNLINK => regs.ax = do_sys_unlink(regs.bx as *const u8),
        SYS_WAITPID => regs.ax = do_sys_waitpid(regs.bx as isize, regs.cx as *mut usize, regs.dx),
        SYS_WRITE => regs.ax = do_sys_write(regs.bx, regs.cx as *mut u8, regs.dx),
        SYS_YIELD => do_sys_yield(),

        // Rust Memory
        SYS_ALLOC => regs.ax = do_sys_alloc(regs.bx),
        SYS_REALLOC => regs.ax = do_sys_realloc(regs.bx, regs.cx),
        SYS_REALLOC_INPLACE => regs.ax = do_sys_realloc_inplace(regs.bx, regs.cx),
        SYS_UNALLOC => do_sys_unalloc(regs.bx),

        _ => regs.ax = Error::mux(Err(Error::new(ENOSYS))),
    }
}
