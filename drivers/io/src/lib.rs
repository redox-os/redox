//! I/O functions

#![feature(asm)]
#![feature(const_fn)]
#![feature(core_intrinsics)]
#![no_std]

pub use self::io::*;
pub use self::mmio::*;
pub use self::pio::*;

mod io;
mod mmio;
mod pio;
