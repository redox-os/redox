#![crate_name="redox"]
#![crate_type="rlib"]
#![feature(alloc)]
#![feature(asm)]
#![feature(box_syntax)]
#![feature(core_slice_ext)]
#![feature(core_str_ext)]
#![feature(lang_items)]
#![feature(no_std)]
#![no_std]

extern crate alloc;

pub use alloc::boxed::*;

pub use common::event::*;
pub use common::queue::*;
pub use common::random::*;
pub use common::string::*;
pub use common::time::*;
pub use common::vec::*;

pub use externs::*;

pub use syscall::call::*;

pub use audio::wav::*;
pub use console::*;
pub use env::*;
pub use file::*;
pub use graphics::bmp::*;
pub use orbital::*;

/// A module for audio
mod audio {
    pub mod wav;
}

/// A module for common functionalities.
/// Primary functionality provided by std.
#[path="../../src/common/src/lib.rs"]
mod common;

#[path="../../src/externs.rs"]
pub mod externs;

/// A module for system calls
#[path="../../src/syscall/src"]
mod syscall {
    /// Calls
    pub mod call;
    /// Common
    pub mod common;
}

/// A module for audio
mod audio {
    pub mod wav;
}

/// A module for console functionality
#[macro_use]
pub mod console;
/// A module for commands and enviroment
pub mod env;
/// A module for the filesystem
pub mod file;
/// Graphics support
mod graphics {
    pub mod bmp;
}
/// A module for window support
pub mod orbital;
