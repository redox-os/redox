pub const SYS_DEBUG: u32 = 0;

//Linux compatible
pub const SYS_EXIT: u32 = 1;
pub const SYS_FORK: u32 = 2;
pub const SYS_READ: u32 = 3;
pub const SYS_WRITE: u32 = 4;
pub const SYS_OPEN: u32 = 5;
pub const SYS_CLOSE: u32 = 6;
pub const SYS_LSEEK: u32 = 19;
pub const SYS_FSTAT: u32 = 28;
pub const SYS_BRK: u32 = 45;
pub const SYS_GETTIMEOFDAY: u32 = 78;
pub const SYS_FSYNC: u32 = 118;
pub const SYS_YIELD: u32 = 158;

//Rust Memory
pub const SYS_ALLOC: u32 = 1000;
pub const SYS_REALLOC: u32 = 1001;
pub const SYS_REALLOC_INPLACE: u32 = 1002;
pub const SYS_UNALLOC: u32 = 1003;

//Windowing
pub const SYS_TRIGGER: u32 = 2000;

//Misc
pub const SYS_TIME: u32 = 3000;
pub const SYS_FPATH: u32 = 3001;
