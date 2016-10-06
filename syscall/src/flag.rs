pub const CLONE_VM: usize = 0x100;
pub const CLONE_FS: usize = 0x200;
pub const CLONE_FILES: usize = 0x400;
pub const CLONE_VFORK: usize = 0x4000;
/// Mark this clone as supervised.
///
/// This means that the process can run in supervised mode, even not being connected to
/// a supervisor yet. In other words, the parent can later on supervise the process and handle
/// the potential blocking syscall.
///
/// This is an important security measure, since otherwise the process would be able to fork it
/// self right after starting, making supervising it impossible.
pub const CLONE_SUPERVISE: usize = 0x400000;
pub const CLOCK_REALTIME: usize = 1;
pub const CLOCK_MONOTONIC: usize = 4;

pub const EVENT_NONE: usize = 0;
pub const EVENT_READ: usize = 1;
pub const EVENT_WRITE: usize = 2;

pub const FUTEX_WAIT: usize = 0;
pub const FUTEX_WAKE: usize = 1;
pub const FUTEX_REQUEUE: usize = 2;

pub const MAP_WRITE: usize = 1;
pub const MAP_WRITE_COMBINE: usize = 2;

pub const MODE_TYPE: u16 = 0xF000;
pub const MODE_DIR: u16 = 0x4000;
pub const MODE_FILE: u16 = 0x8000;

pub const MODE_PERM: u16 = 0x0FFF;
pub const MODE_SETUID: u16 = 0o4000;
pub const MODE_SETGID: u16 = 0o2000;

pub const SEEK_SET: usize = 0;
pub const SEEK_CUR: usize = 1;
pub const SEEK_END: usize = 2;

pub const O_RDONLY: usize =     0x0000_0000;
pub const O_WRONLY: usize =     0x0001_0000;
pub const O_RDWR: usize =       0x0002_0000;
pub const O_NONBLOCK: usize =   0x0004_0000;
pub const O_APPEND: usize =     0x0008_0000;
pub const O_SHLOCK: usize =     0x0010_0000;
pub const O_EXLOCK: usize =     0x0020_0000;
pub const O_ASYNC: usize =      0x0040_0000;
pub const O_FSYNC: usize =      0x0080_0000;
pub const O_CLOEXEC: usize =    0x0100_0000;
pub const O_CREAT: usize =      0x0200_0000;
pub const O_TRUNC: usize =      0x0400_0000;
pub const O_EXCL: usize =       0x0800_0000;

pub const WNOHANG: usize = 1;
