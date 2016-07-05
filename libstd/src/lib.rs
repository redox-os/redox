//! # The Redox Library
//!
//! The Redox Library contains a collection of commonly used low-level software
//! constructs to be used on top of the base operating system, including graphics
//! support and windowing, a basic filesystem, audio support, a simple console
//! with shell style functions, an event system, and environment argument support.

#![crate_name="std"]
#![crate_type="rlib"]
#![feature(alloc)]
#![feature(allow_internal_unstable)]
#![feature(asm)]
#![feature(associated_consts)]
#![feature(box_patterns)]
#![feature(box_syntax)]
#![feature(cfg_target_thread_local)]
#![feature(collections)]
#![feature(collections_bound)]
#![feature(const_fn)]
#![feature(core_intrinsics)]
#![feature(core_panic)]
#![feature(dropck_parametricity)]
#![feature(filling_drop)]
#![feature(float_extras)]
#![feature(heap_api)]
#![feature(int_error_internals)]
#![feature(lang_items)]
#![feature(macro_reexport)]
#![feature(naked_functions)]
#![feature(oom)]
#![feature(prelude_import)]
#![feature(question_mark)]
#![feature(rand)]
#![feature(raw)]
#![feature(reflect_marker)]
#![feature(slice_concat_ext)]
#![feature(slice_patterns)]
#![feature(stmt_expr_attributes)]
#![feature(type_ascription)]
#![feature(unicode)]
#![feature(unique)]
#![feature(unsafe_no_drop_flag)]
#![no_std]

#![allow(deprecated)]
// TODO
//#![deny(missing_docs)]
#![deny(warnings)]

// Bring in memcpy, memcmp, memmove, memset
pub mod externs;
pub use externs::*;

extern crate ralloc;

// STD COPY {
// We want to reexport a few macros from core but libcore has already been
// imported by the compiler (via our #[no_std] attribute) In this case we just
// add a new crate name so we can attach the reexports to it.
#[macro_reexport(assert, assert_eq, debug_assert, debug_assert_eq,
                    unreachable, unimplemented, write, writeln)]
extern crate core as __core;

#[macro_use]
#[macro_reexport(vec, format)]
extern crate collections as core_collections;

#[allow(deprecated)]
pub extern crate rand as core_rand;
extern crate alloc;
extern crate rustc_unicode;
// TODO extern crate libc;

extern crate system;

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
pub use core::result;
pub use core::option;
pub mod error;

pub use alloc::arc;
pub use alloc::boxed;
pub use alloc::rc;

pub use core_collections::borrow;
pub use core_collections::fmt;
pub use core_collections::slice;
pub use core_collections::str;
pub use core_collections::string;
pub use core_collections::vec;

pub use rustc_unicode::char;

// Exported macros

#[macro_use]
pub mod macros;

// TODO mod rtdeps;

// The Prelude.
#[prelude_import]
pub mod prelude;

// Primitive types

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

pub mod ascii;

// Common traits

//pub mod num;
pub use core::num;
#[path = "num/f32.rs"]   pub mod f32;
#[path = "num/f64.rs"]   pub mod f64;

// Runtime and platform support

#[macro_use]
pub mod thread;

pub mod collections;
// TODO pub mod dynamic_lib;
pub mod env;
pub mod ffi;
pub mod fs;
pub mod io;
pub mod net;
pub mod os;
pub mod path;
pub mod process;
pub mod sync;
pub mod time;

#[macro_use]
#[path = "sys/common/mod.rs"] mod sys_common;

// TODO #[cfg(unix)]
// TODO #[path = "sys/unix/mod.rs"] mod sys;
// TODO #[cfg(windows)]
// TODO #[path = "sys/windows/mod.rs"] mod sys;

pub mod rt;
// TODO mod panicking;
pub use __core::panicking;

pub mod rand_old;

// Some external utilities of the standard library rely on randomness (aka
// rustc_back::TempDir and tests) and need a way to get at the OS rng we've got
// here. This module is not at all intended for stabilization as-is, however,
// but it may be stabilized long-term. As a result we're exposing a hidden,
// unstable module so we can get our build working.
    #[doc(hidden)]
// TODO #[unstable(feature = "rand", issue = "0")]
pub use core_rand as rand;
// } STD COPY

pub use rand_old::*;

pub mod panic;

pub mod to_num;
