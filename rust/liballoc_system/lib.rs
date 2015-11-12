// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![cfg_attr(stage0, feature(custom_attribute))]
#![crate_name = "alloc_system"]
#![crate_type = "rlib"]
#![staged_api]
#![no_std]
#![cfg_attr(not(stage0), allocator)]
#![unstable(feature = "alloc_system",
            reason = "this library is unlikely to be stabilized in its current \
                      form or name",
            issue = "27783")]
#![feature(allocator)]
#![feature(asm)]
#![feature(no_std)]
#![feature(staged_api)]


// The minimum alignment guaranteed by the architecture. This value is used to
// add fast paths for low alignment values. In practice, the alignment is a
// constant at the call site and the branch will be optimized out.
#[cfg(all(any(target_arch = "arm",
              target_arch = "mips",
              target_arch = "mipsel",
              target_arch = "powerpc")))]
const MIN_ALIGN: usize = 8;
#[cfg(all(any(target_arch = "x86",
              target_arch = "x86_64",
              target_arch = "aarch64")))]
const MIN_ALIGN: usize = 16;

extern {
    fn memmove(dst: *mut u8, src: *const u8, size: usize);
    fn __rust_allocate(size: usize, align: usize) -> *mut u8;
    fn __rust_deallocate(ptr: *mut u8, old_size: usize, align: usize);
    fn __rust_reallocate(ptr: *mut u8, old_size: usize, size: usize, align: usize) -> *mut u8;
    fn __rust_reallocate_inplace(ptr: *mut u8, old_size: usize, size: usize, align: usize) -> usize;
    fn __rust_usable_size(size: usize, align: usize) -> usize;
 }
