#![crate_name="redox"]
#![crate_type="rlib"]
#![feature(alloc)]
#![feature(asm)]
#![feature(box_syntax)]
#![feature(core_simd)]
#![feature(core_slice_ext)]
#![feature(core_str_ext)]
#![feature(lang_items)]
#![feature(no_std)]
#![no_std]

extern crate alloc;

pub use alloc::boxed::*;

pub use audio::wav::*;

pub use common::event::*;
pub use common::queue::*;
pub use common::random::*;
pub use common::string::*;
pub use common::time::*;
pub use common::vec::*;

pub use graphics::bmp::*;
pub use graphics::color::*;
pub use graphics::display::*;
pub use graphics::point::*;
pub use graphics::size::*;
pub use graphics::window::*;

pub use externs::*;

pub use syscall::call::*;

pub use console::*;
pub use env::*;
pub use file::*;
pub use orbital::*;

/// A module for audio
#[path="../../src/audio"]
mod audio {
    pub mod wav;
}

/// A module for common functionalities.
/// Primary functionality provided by std.
#[path="../../src/common/src/lib.rs"]
mod common;

#[path="../../src/externs.rs"]
pub mod externs;

/// A module for graphics
#[path="../../src/graphics/src/lib.rs"]
mod graphics;

/// A module for system calls
#[path="../../src/syscall"]
mod syscall {
    /// Calls
    pub mod call;
    /// Common
    pub mod common;
}

/// A module for console functionality
#[macro_use]
pub mod console;
/// A module for commands and enviroment
pub mod env;
/// A module for the filesystem
pub mod file;
/// A module for window support
pub mod orbital;
