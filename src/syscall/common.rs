//Going to use linux syscall definitions from http://docs.cs.up.ac.za/programming/asm/derick_tut/syscalls.html

pub const SYS_DEBUG: u32 = 0;
pub const SYS_EXIT: u32 = 1;
pub const SYS_OPEN: u32 = 2;
pub const SYS_TRIGGER: u32 = 3;
pub const SYS_YIELD: u32 = 4;
