#![no_std]
#![allow(non_camel_case_types)]
#![feature(asm)]
#![feature(const_fn)]
#![feature(naked_functions)]
#![feature(thread_local)]

#![cfg_attr(stdbuild, feature(no_std, core, core_slice_ext, staged_api, custom_attribute, cfg_target_vendor))]
#![cfg_attr(stdbuild, no_std)]
#![cfg_attr(stdbuild, staged_api)]
#![cfg_attr(stdbuild, allow(warnings))]
#![cfg_attr(stdbuild, unstable(feature = "libc",
                               reason = "use `libc` from crates.io",
                               issue = "27783"))]

pub use types::*;
pub use funcs::*;
pub use start::*;
pub use syscall::*;

/// Basic types (not usually system specific)
mod types;
/// Basic functions (not system specific)
mod funcs;
/// Start function and call in to libstd
mod start;
/// Conversion for syscall library (specific to Redox)
mod syscall;
