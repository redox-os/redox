use syscall::sys_yield;

//TODO: JoinHandle
pub fn spawn<F, T>(f: F) where F: FnOnce() -> T, F: Send + 'static, T: Send + 'static {

}
