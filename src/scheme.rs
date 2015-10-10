#![crate_type="staticlib"]
#![allow(unused_features)]
#![feature(alloc)]
#![feature(asm)]
#![feature(box_syntax)]
#![feature(collections)]
#![feature(convert)]
#![feature(core_slice_ext)]
#![feature(no_std)]
#![feature(vec_push_all)]
#![feature(vec_resize)]
#![no_std]

#[macro_use]
extern crate alloc;

#[macro_use]
extern crate collections;

#[macro_use]
extern crate redox;

use scheme::Resource;
use scheme::Scheme;

#[path="SCHEME_PATH"]
mod scheme;

use redox::Box;
use redox::fs::file::Seek;
use redox::slice;

#[no_mangle]
pub unsafe extern "C" fn _start() -> *mut Scheme {
    Box::into_raw(Box::new(Scheme::new()))
}

#[no_mangle]
pub unsafe extern "C" fn _stop(scheme: *mut Scheme) {
    drop(Box::from_raw(scheme));
}

#[no_mangle]
pub unsafe extern "C" fn _open(scheme: *mut Scheme, path: &str) -> *mut Resource {
    Box::into_raw(Box::new((*scheme).open(path)))
}

#[no_mangle]
pub unsafe extern "C" fn _read(resource: *mut Resource, buf: *mut u8, len: usize) -> usize {
    match (*resource).read(slice::from_raw_parts_mut(buf, len)) {
        Some(bytes) => return bytes,
        None => return 0xFFFFFFFF
    }
}

#[no_mangle]
pub unsafe extern "C" fn _write(resource: *mut Resource, buf: *const u8, len: usize) -> usize {
    match (*resource).write(slice::from_raw_parts(buf, len)) {
        Some(bytes) => return bytes,
        None => return 0xFFFFFFFF
    }
}

const SEEK_SET: isize = 0;
const SEEK_CUR: isize = 1;
const SEEK_END: isize = 2;
#[no_mangle]
pub unsafe extern "C" fn _lseek(resource: *mut Resource, offset: isize, whence: isize) -> usize {
    if whence == SEEK_SET {
        if let Some(bytes) = (*resource).seek(Seek::Start(offset as usize)) {
            return bytes;
        }
    } else if whence == SEEK_CUR {
        if let Some(bytes) = (*resource).seek(Seek::Current(offset)) {
            return bytes;
        }
    } else if whence == SEEK_END {
        if let Some(bytes) = (*resource).seek(Seek::End(offset)) {
            return bytes;
        }
    }

    0xFFFFFFFF
}

#[no_mangle]
pub unsafe extern "C" fn _fsync(resource: *mut Resource) -> usize {
    if (*resource).sync() {
        0
    } else {
        0xFFFFFFFF
    }
}

#[no_mangle]
pub unsafe extern "C" fn _close(resource: *mut Resource) -> usize {
    drop(Box::from_raw(resource));
    0
}
