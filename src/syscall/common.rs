//Going to use linux syscall definitions from http://docs.cs.up.ac.za/programming/asm/derick_tut/syscalls.html

pub const SYS_DEBUG: u32 = 0;
pub const SYS_EXIT: u32 = 1;
pub const SYS_OPEN: u32 = 2;
pub const SYS_TCP_LISTENER_CREATE: u32 = 3;
pub const SYS_TCP_LISTENER_DESTROY: u32 = 4;
pub const SYS_TRIGGER: u32 = 5;
pub const SYS_WINDOW_CREATE: u32 = 6;
pub const SYS_WINDOW_DESTROY: u32 = 7;
pub const SYS_YIELD: u32 = 8;
