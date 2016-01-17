// Copyright 2013 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! The first version of the prelude of The Rust Standard Library.

// Reexported core operators
#[doc(no_inline)]
pub use marker::{Copy, Send, Sized, Sync};
#[doc(no_inline)]
pub use ops::{Drop, Fn, FnMut, FnOnce};

// Reexported functions
#[doc(no_inline)]
pub use mem::drop;

// Reexported types and traits
#[doc(no_inline)]
pub use boxed::Box;
#[doc(no_inline)]
pub use borrow::ToOwned;
#[doc(no_inline)]
pub use clone::Clone;
#[doc(no_inline)]
pub use cmp::{PartialEq, PartialOrd, Eq, Ord};
#[doc(no_inline)]
pub use convert::{AsRef, AsMut, Into, From};
#[doc(no_inline)]
pub use default::Default;
#[doc(no_inline)]
pub use iter::{Iterator, Extend, IntoIterator};
#[doc(no_inline)]
pub use iter::{DoubleEndedIterator, ExactSizeIterator};
#[doc(no_inline)]
pub use option::Option::{self, Some, None};
#[doc(no_inline)]
pub use result::Result::{self, Ok, Err};
#[doc(no_inline)]
pub use slice::SliceConcatExt;
#[doc(no_inline)]
pub use string::{String, ToString};
#[doc(no_inline)]
pub use vec::Vec;
