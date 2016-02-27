pub use self::arch::*;
pub use self::unix::*;
pub use self::redox::*;

// Unix compatible
pub mod unix;

// Redox special
pub mod redox;

#[cfg(target_arch = "x86")]
#[path="x86.rs"]
pub mod arch;

#[cfg(target_arch = "x86_64")]
#[path="x86_64.rs"]
pub mod arch;
