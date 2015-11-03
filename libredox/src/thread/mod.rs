use syscall::{sys_clone, sys_exit};
use syscall::common::{CLONE_VM, CLONE_FS, CLONE_FILES};

//TODO: JoinHandle
pub fn spawn<F, T>(f: F) where F: FnOnce() -> T, F: Send + 'static, T: Send + 'static {
    if unsafe { sys_clone(CLONE_VM | CLONE_FS | CLONE_FILES) } == 0 {
        f();
        unsafe { sys_exit(0) };
    }
}
