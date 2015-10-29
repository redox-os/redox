use super::syscall::sys_debug;

pub fn debug<T: AsRef<str>>(msg: T) {
    for b in msg.as_ref().bytes() {
        unsafe {
            sys_debug(b);
        }
    }
}
