pub use self::arch::*;

#[cfg(target_arch = "x86")]
#[path="x86/gdt.rs"]
mod arch;

#[cfg(target_arch = "x86_64")]
#[path="x86_64/gdt.rs"]
mod arch;
