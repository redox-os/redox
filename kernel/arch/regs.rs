pub use self::arch::*;

#[cfg(target_arch = "x86")]
#[path="x86/regs.rs"]
mod arch;

#[cfg(target_arch = "x86_64")]
#[path="x86_64/regs.rs"]
mod arch;
