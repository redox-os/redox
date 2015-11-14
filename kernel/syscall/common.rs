pub const SYS_DEBUG: usize = 0;

// Linux compatible
pub const SYS_BRK: usize = 45;
pub const SYS_CHDIR: usize = 12;
pub const SYS_CLOSE: usize = 6;
pub const SYS_CLONE: usize = 120;
    pub const CLONE_VM: usize = 0x100;
    pub const CLONE_FS: usize = 0x200;
    pub const CLONE_FILES: usize = 0x400;
pub const SYS_CLOCK_GETTIME: usize = 265;
    pub const CLOCK_REALTIME: usize = 0;
    pub const CLOCK_MONOTONIC: usize = 1;
pub const SYS_DUP: usize = 41;
pub const SYS_EXECVE: usize = 11;
pub const SYS_EXIT: usize = 1;
pub const SYS_FPATH: usize = 3001;
pub const SYS_FSTAT: usize = 28;
pub const SYS_FSYNC: usize = 118;
pub const SYS_FTRUNCATE: usize = 93;
pub const SYS_LINK: usize = 9;
pub const SYS_LSEEK: usize = 19;
    pub const SEEK_SET: usize = 0;
    pub const SEEK_CUR: usize = 1;
    pub const SEEK_END: usize = 2;
pub const SYS_MKDIR: usize = 39;
pub const SYS_NANOSLEEP: usize = 162;
pub const SYS_OPEN: usize = 5;
    pub const O_RDONLY: usize = 0;
    pub const O_WRONLY: usize = 1;
    pub const O_RDWR: usize = 2;
    pub const O_NONBLOCK: usize = 4;
    pub const O_APPEND: usize = 8;
    pub const O_SHLOCK: usize = 0x10;
    pub const O_EXLOCK: usize = 0x20;
    pub const O_ASYNC: usize = 0x40;
    pub const O_FSYNC: usize = 0x80;
    pub const O_CREAT: usize = 0x200;
    pub const O_TRUNC: usize = 0x400;
    pub const O_EXCL: usize = 0x800;
pub const SYS_READ: usize = 3;
pub const SYS_UNLINK: usize = 10;
pub const SYS_WRITE: usize = 4;
pub const SYS_YIELD: usize = 158;

// Rust Memory
pub const SYS_ALLOC: usize = 1000;
pub const SYS_REALLOC: usize = 1001;
pub const SYS_REALLOC_INPLACE: usize = 1002;
pub const SYS_UNALLOC: usize = 1003;

// Structures

#[repr(packed)]
pub struct TimeSpec {
    pub tv_sec: i64,
    pub tv_nsec: i32,
}
