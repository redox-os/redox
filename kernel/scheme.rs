#![crate_type="staticlib"]
#![allow(unused_features)]
#![feature(asm)]
#![feature(box_syntax)]
#![feature(no_std)]
#![feature(vec_push_all)]
#![feature(vec_resize)]
#![no_std]

#[macro_use]
extern crate redox;

extern crate orbital;

use scheme::{Resource, Scheme};

#[path="SCHEME_PATH"]
mod scheme;

use redox::Box;
use redox::io::{Read, Write, Seek, SeekFrom};
use redox::{ptr, slice, str, usize};

#[cold]
#[inline(never)]
#[no_mangle]
pub unsafe extern "C" fn _start() -> *mut Scheme {
    Box::into_raw(Scheme::new())
}

#[cold]
#[inline(never)]
#[no_mangle]
pub unsafe extern "C" fn _stop(scheme: *mut Scheme) {
    drop(Box::from_raw(scheme));
}

#[cold]
#[inline(never)]
#[no_mangle]
pub unsafe extern "C" fn _open(scheme: *mut Scheme, path: *const u8, flags: usize) -> *mut Resource {
    let mut len = 0;
    for i in 0..4096 {
        len = i as usize;
        if ptr::read(path.offset(i)) == 0 {
            break;
        }
    }

    match (*scheme).open(str::from_utf8_unchecked(slice::from_raw_parts(path, len)), flags) {
        Some(resource) => return Box::into_raw(resource),
        None => return usize::MAX as *mut Resource
    }
}


#[cold]
#[inline(never)]
#[no_mangle]
pub unsafe extern "C" fn _dup(resource: *mut Resource) -> *mut Resource {
    match (*resource).dup() {
        Some(resource) => return Box::into_raw(resource),
        None => return usize::MAX as *mut Resource
    }
}

#[cold]
#[inline(never)]
#[no_mangle]
pub unsafe extern "C" fn _fpath(resource: *mut Resource, buf: *mut u8, len: usize) -> usize {
    match (*resource).path() {
        Some(string) => {
            let mut buf = slice::from_raw_parts_mut(buf, len);

            let mut i = 0;
            for b in string.bytes() {
                if i < buf.len() {
                    buf[i] = b;
                    i += 1;
                } else {
                    break;
                }
            }

            return i;
        },
        None => return usize::MAX
    }
}

#[cold]
#[inline(never)]
#[no_mangle]
pub unsafe extern "C" fn _read(resource: *mut Resource, buf: *mut u8, len: usize) -> usize {
    match (*resource).read(slice::from_raw_parts_mut(buf, len)) {
        Some(bytes) => return bytes,
        None => return usize::MAX
    }
}

#[cold]
#[inline(never)]
#[no_mangle]
pub unsafe extern "C" fn _write(resource: *mut Resource, buf: *const u8, len: usize) -> usize {
    match (*resource).write(slice::from_raw_parts(buf, len)) {
        Some(bytes) => return bytes,
        None => return usize::MAX
    }
}

const SEEK_SET: isize = 0;
const SEEK_CUR: isize = 1;
const SEEK_END: isize = 2;
#[cold]
#[inline(never)]
#[no_mangle]
pub unsafe extern "C" fn _lseek(resource: *mut Resource, offset: isize, whence: isize) -> usize {
    if whence == SEEK_SET {
        if let Some(bytes) = (*resource).seek(SeekFrom::Start(offset as usize)) {
            return bytes;
        }
    } else if whence == SEEK_CUR {
        if let Some(bytes) = (*resource).seek(SeekFrom::Current(offset)) {
            return bytes;
        }
    } else if whence == SEEK_END {
        if let Some(bytes) = (*resource).seek(SeekFrom::End(offset)) {
            return bytes;
        }
    }

    usize::MAX
}

#[cold]
#[inline(never)]
#[no_mangle]
pub unsafe extern "C" fn _fsync(resource: *mut Resource) -> usize {
    if (*resource).sync() {
        0
    } else {
        usize::MAX
    }
}

#[cold]
#[inline(never)]
#[no_mangle]
pub unsafe extern "C" fn _close(resource: *mut Resource) -> usize {
    drop(Box::from_raw(resource));
    0
}
