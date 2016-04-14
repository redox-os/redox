// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

pub use hash::BuildHasher;

use clone::Clone;
use default::Default;
use hash;
use marker;

/// A structure which is a factory for instances of `Hasher` which implement the
/// default trait.
///
/// This struct is 0-sized and does not need construction.
pub struct DefaultState<H>(marker::PhantomData<H>);

impl<H: Default + hash::Hasher> BuildHasher for DefaultState<H> {
    type Hasher = H;
    fn build_hasher(&self) -> H { Default::default() }
}

impl<H> Clone for DefaultState<H> {
    fn clone(&self) -> DefaultState<H> { DefaultState(marker::PhantomData) }
}

impl<H> Default for DefaultState<H> {
    fn default() -> DefaultState<H> { DefaultState(marker::PhantomData) }
}
