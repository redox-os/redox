//! # The Redox Library
//!
//! The Redox Library contains a collection of commonly used low-level software
//! constructs to be used on top of the base operating system, including graphics
//! support and windowing, a basic filesystem, audio support, a simple console
//! with shell style functions, an event system, and environment argument support.

#![crate_type="rlib"]
#![feature(alloc)]
#![feature(allocator)]
#![feature(allow_internal_unstable)]
#![feature(asm)]
#![feature(associated_consts)]
#![feature(box_syntax)]
#![feature(collections)]
#![feature(collections_bound)]
#![feature(core)]
#![feature(core_intrinsics)]
#![feature(core_panic)]
#![feature(core_simd)]
#![feature(int_error_internals)]
#![feature(lang_items)]
#![feature(macro_reexport)]
#![feature(rand)]
#![feature(raw)]
#![feature(reflect_marker)]
#![feature(slice_concat_ext)]
#![feature(unicode)]
#![feature(unsafe_no_drop_flag)]
#![feature(vec_push_all)]
#![feature(wrapping)]
#![feature(zero_one)]
#![feature(no_std)]
#![no_std]

//#![warn(missing_docs)]

/* STD COPY { */
    // We want to reexport a few macros from core but libcore has already been
    // imported by the compiler (via our #[no_std] attribute) In this case we just
    // add a new crate name so we can attach the reexports to it.
    #[macro_reexport(assert, assert_eq, debug_assert, debug_assert_eq,
                    unreachable, unimplemented, write, writeln)]
    extern crate core as __core;

    #[macro_use]
    #[macro_reexport(vec, format)]
    extern crate collections as core_collections;

    #[allow(deprecated)] extern crate rand as core_rand;
    extern crate alloc;
    extern crate rustc_unicode;
    //TODO extern crate libc;

    // NB: These reexports are in the order they should be listed in rustdoc

    pub use core::any;
    pub use core::cell;
    pub use core::clone;
    pub use core::cmp;
    pub use core::convert;
    pub use core::default;
    pub use core::hash;
    pub use core::intrinsics;
    pub use core::iter;
    pub use core::marker;
    pub use core::mem;
    pub use core::ops;
    pub use core::ptr;
    pub use core::raw;
    #[allow(deprecated)]
    pub use core::simd;
    pub use core::result;
    pub use core::option;
    pub mod error;
    pub mod debug;

    pub use alloc::boxed;
    pub use alloc::rc;

    pub use core_collections::borrow;
    pub use core_collections::fmt;
    pub use core_collections::slice;
    pub use core_collections::str;
    pub use core_collections::string;
    pub use core_collections::vec;

    pub use rustc_unicode::char;

    /* Exported macros */

    #[cfg(std)]
    #[macro_use]
    mod macros;

    // TODO mod rtdeps;

    /* The Prelude. */
    pub mod prelude;


    /* Primitive types */

    // NB: slice and str are primitive types too, but their module docs + primitive
    // doc pages are inlined from the public re-exports of core_collections::{slice,
    // str} above.

    pub use core::isize;
    pub use core::i8;
    pub use core::i16;
    pub use core::i32;
    pub use core::i64;

    pub use core::usize;
    pub use core::u8;
    pub use core::u16;
    pub use core::u32;
    pub use core::u64;

    // TODO: Add methods to f64
    pub use core::num;

    //TODO #[path = "num/f32.rs"]   pub mod f32;
    //TODO #[path = "num/f64.rs"]   pub mod f64;

    pub mod ascii;

    /* Common traits */

    pub mod floating_num;

    /* Runtime and platform support */

    // TODO #[macro_use]
    // TODO pub mod thread;

    pub mod collections;
    // TODO pub mod dynamic_lib;
    pub mod env;
    // TODO pub mod ffi;
    pub mod fs;
    pub mod io;
    pub mod net;
    pub mod package;
    // TODO pub mod os;
    // TODO pub mod path;
    // TODO pub mod process;
    // TODO pub mod sync;
    pub mod time;

    //TODO #[macro_use]
    //TODO #[path = "sys/common/mod.rs"] mod sys_common;

    //TODO #[cfg(unix)]
    //TODO #[path = "sys/unix/mod.rs"] mod sys;
    //TODO #[cfg(windows)]
    //TODO #[path = "sys/windows/mod.rs"] mod sys;

    #[cfg(std)]
    pub mod rt;
    //TODO mod panicking;
    pub use __core::panicking;

    mod rand_old;

    // Some external utilities of the standard library rely on randomness (aka
    // rustc_back::TempDir and tests) and need a way to get at the OS rng we've got
    // here. This module is not at all intended for stabilization as-is, however,
    // but it may be stabilized long-term. As a result we're exposing a hidden,
    // unstable module so we can get our build working.
    #[doc(hidden)]
    //TODO #[unstable(feature = "rand", issue = "0")]
    pub mod rand {
        pub use core_rand::{/*thread_rng, ThreadRng,*/ Rng};
    }
/* } STD COPY */

/* Additional Stuff { */
    pub use boxed::Box;
    pub use env::*;
    pub use fs::*;
    pub use io::*;
    pub use rand_old::*;
    pub use string::*;
    pub use vec::Vec;

    pub use audio::wav::*;
    #[cfg(not(std))]
    pub use console::*;
    pub use graphics::bmp::*;
    pub use graphics::color::*;
    pub use graphics::size::*;
    pub use graphics::point::*;
    pub use graphics::display::*;
    pub use orbital::*;
    pub use orbital::event::*;
    pub use orbital::session::*;
    pub use orbital::window::*;
    pub use url::*;
    pub use to_num::*;

    pub mod alloc_system;

    /// A module for necessary C and assembly constructs
    #[path="../../kernel/externs.rs"]
    pub mod externs;

    pub mod panic;

    /// A module for system calls
    pub mod syscall;

    /// A module for audio
    mod audio {
        pub mod wav;
    }

    /// A module for console functionality
    #[cfg(not(std))]
    #[macro_use]
    pub mod console;
    /// Graphics support
    mod graphics {
        pub mod bmp;
        pub mod color;
        pub mod point;
        pub mod size;
        pub mod display;
    }
    /// A module for window support
    pub mod orbital {
        pub mod session;
        pub mod window;
        pub mod event;
    }

    pub mod url;

    pub mod to_num;
/* } Additional Stuff */
