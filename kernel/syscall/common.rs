pub const SYS_DEBUG: usize = 0;

//Linux compatible
pub const SYS_BRK: usize = 45;
pub const SYS_CHDIR: usize = 12;
pub const SYS_CLOSE: usize = 6;
pub const SYS_DUP: usize = 41;
pub const SYS_EXECVE: usize = 11;
pub const SYS_EXIT: usize = 1;
pub const SYS_FORK: usize = 2;
pub const SYS_FPATH: usize = 3001;
pub const SYS_FSTAT: usize = 28;
pub const SYS_FSYNC: usize = 118;
pub const SYS_GETTIMEOFDAY: usize = 78;
pub const SYS_LINK: usize = 9;
pub const SYS_LSEEK: usize = 19;
pub const SYS_OPEN: usize = 5;
pub const SYS_READ: usize = 3;
pub const SYS_UNLINK: usize = 10;
pub const SYS_WRITE: usize = 4;
pub const SYS_YIELD: usize = 158;

//Rust Memory
pub const SYS_ALLOC: usize = 1000;
pub const SYS_REALLOC: usize = 1001;
pub const SYS_REALLOC_INPLACE: usize = 1002;
pub const SYS_UNALLOC: usize = 1003;
