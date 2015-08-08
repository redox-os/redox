// Copyright 2014-2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use common::memory;

// TODO use lib.rs
use common::debug::*;
pub fn oom() -> ! {
    // FIXME(#14674): This really needs to do something other than just abort
    //                here, but any printing done must be *guaranteed* to not
    //                allocate.
    unsafe {
        d("OOM\n");
        asm!("cli");
        asm!("hlt");

        loop{}
    }
}

// FIXME: #13996: mark the `allocate` and `reallocate` return value as `noalias`

/// Return a pointer to `size` bytes of memory aligned to `align`.
///
/// On failure, return a null pointer.
///
/// Behavior is undefined if the requested size is 0 or the alignment is not a
/// power of 2. The alignment must be no larger than the largest supported page
/// size on the platform.
#[allow(unused_variables)]
#[inline]
pub unsafe fn allocate(size: usize, align: usize) -> *mut u8 {
    return memory::alloc(size) as *mut u8;
}

/// Resize the allocation referenced by `ptr` to `size` bytes.
///
/// On failure, return a null pointer and leave the original allocation intact.
///
/// If the allocation was relocated, the memory at the passed-in pointer is
/// undefined after the call.
///
/// Behavior is undefined if the requested size is 0 or the alignment is not a
/// power of 2. The alignment must be no larger than the largest supported page
/// size on the platform.
///
/// The `old_size` and `align` parameters are the parameters that were used to
/// create the allocation referenced by `ptr`. The `old_size` parameter may be
/// any value in range_inclusive(requested_size, usable_size).
#[allow(unused_variables)]
#[inline]
pub unsafe fn reallocate(ptr: *mut u8, old_size: usize, size: usize, align: usize) -> *mut u8 {
    return memory::realloc(ptr as usize, size) as *mut u8;
}

/// Resize the allocation referenced by `ptr` to `size` bytes.
///
/// If the operation succeeds, it returns `usable_size(size, align)` and if it
/// fails (or is a no-op) it returns `usable_size(old_size, align)`.
///
/// Behavior is undefined if the requested size is 0 or the alignment is not a
/// power of 2. The alignment must be no larger than the largest supported page
/// size on the platform.
///
/// The `old_size` and `align` parameters are the parameters that were used to
/// create the allocation referenced by `ptr`. The `old_size` parameter may be
/// any value in range_inclusive(requested_size, usable_size).
/*
#[inline]
pub unsafe fn reallocate_inplace(ptr: *mut u8, old_size: usize, size: usize,
                                 align: usize) -> usize {
    check_size_and_alignment(size, align);
    imp::reallocate_inplace(ptr, old_size, size, align)
}
*/

/// Deallocates the memory referenced by `ptr`.
///
/// The `ptr` parameter must not be null.
///
/// The `old_size` and `align` parameters are the parameters that were used to
/// create the allocation referenced by `ptr`. The `old_size` parameter may be
/// any value in range_inclusive(requested_size, usable_size).
#[allow(unused_variables)]
#[inline]
pub unsafe fn deallocate(ptr: *mut u8, old_size: usize, align: usize) {
    memory::unalloc(ptr as usize);
}

/// Returns the usable size of an allocation created with the specified the
/// `size` and `align`.
/*
#[inline]
pub fn usable_size(size: usize, align: usize) -> usize {
    imp::usable_size(size, align)
}
*/

/// An arbitrary non-null address to represent zero-size allocations.
///
/// This preserves the non-null invariant for types like `Box<T>`. The address may overlap with
/// non-zero-size memory allocations.
pub const EMPTY: *mut () = 0x1 as *mut ();

/// The allocator for unique pointers.
#[cfg(not(test))]
#[lang = "exchange_malloc"]
#[inline]
unsafe fn exchange_malloc(size: usize, align: usize) -> *mut u8 {
    if size == 0 {
        EMPTY as *mut u8
    } else {
        let ptr = allocate(size, align);
        if ptr.is_null() { oom() }
        ptr
    }
}

#[cfg(not(test))]
#[lang = "exchange_free"]
#[inline]
unsafe fn exchange_free(ptr: *mut u8, old_size: usize, align: usize) {
    deallocate(ptr, old_size, align);
}
