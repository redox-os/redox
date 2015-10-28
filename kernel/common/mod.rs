/// Context
pub mod context;
/// Debug
pub mod debug;
/// ELF File Support
pub mod elf;
/// Event input
pub mod event;
/// Kernel memory allocation
pub mod memory;
/// Paging (x86)
#[cfg(target_arch = "x86")]
#[path="paging-i386.rs"]
pub mod paging;
/// Paging (x86_64)
#[cfg(target_arch = "x86_64")]
#[path="paging-x86_64.rs"]
pub mod paging;
/// A module for queues
pub mod queue;
/// A module for pseudorandom generator
pub mod random;
/// A module for scheduling
pub mod scheduler;
/// A module for time
pub mod time;
/// String to number
pub mod to_num;
/// A module for parsing paths
pub mod parse_path;
/// A module for parsing IP related string 
pub mod parse_ip;
