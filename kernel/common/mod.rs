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
/// A module for owned strings
pub mod string;
/// A module for time
pub mod time;
/// A module for heap allocated, growable vectors
pub mod vec;
