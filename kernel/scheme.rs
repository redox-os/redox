#![crate_type="staticlib"]
#![allow(unused_features)]
#![feature(asm)]
#![feature(box_syntax)]
#![feature(vec_push_all)]
#![feature(vec_resize)]

extern crate orbital;

use scheme::{Resource, Scheme};

#[path="SCHEME_PATH"]
pub mod scheme;

use std::io::{Read, Write, Seek, SeekFrom};
use std::{ptr, slice, str};
use std::syscall::{SysError, EINVAL};

#[no_mangle]
pub fn main(){
    println!("This is a scheme, it cannot be run like a normal program");
}

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
pub unsafe extern "C" fn _open(scheme: *mut Scheme, path: *const u8, flags: usize) -> usize {
    let mut len = 0;
    for i in 0..4096 {
        len = i as usize;
        if ptr::read(path.offset(i)) == 0 {
            break;
        }
    }

    SysError::mux(
        match (*scheme).open(str::from_utf8_unchecked(slice::from_raw_parts(path, len)), flags) {
            Ok(resource) => Ok(Box::into_raw(resource) as usize),
            Err(err) => Err(err)
        }
    )
}


#[cold]
#[inline(never)]
#[no_mangle]
pub unsafe extern "C" fn _dup(resource: *mut Resource) -> usize {
    SysError::mux(
        match (*resource).dup() {
            Ok(resource) => Ok(Box::into_raw(resource) as usize),
            Err(err) => Err(err)
        }
    )
}

#[cold]
#[inline(never)]
#[no_mangle]
pub unsafe extern "C" fn _fpath(resource: *mut Resource, buf: *mut u8, len: usize) -> usize {
    SysError::mux(
        match (*resource).path() {
            Ok(string) => {
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

                Ok(i)
            },
            Err(err) => Err(err)
        }
    )
}

#[cold]
#[inline(never)]
#[no_mangle]
pub unsafe extern "C" fn _read(resource: *mut Resource, buf: *mut u8, len: usize) -> usize {
    SysError::mux(
        (*resource).read(slice::from_raw_parts_mut(buf, len))
    )
}

#[cold]
#[inline(never)]
#[no_mangle]
pub unsafe extern "C" fn _write(resource: *mut Resource, buf: *const u8, len: usize) -> usize {
    SysError::mux(
        (*resource).write(slice::from_raw_parts(buf, len))
    )
}

const SEEK_SET: isize = 0;
const SEEK_CUR: isize = 1;
const SEEK_END: isize = 2;
#[cold]
#[inline(never)]
#[no_mangle]
pub unsafe extern "C" fn _lseek(resource: *mut Resource, offset: isize, whence: isize) -> usize {
    let result = if whence == SEEK_SET {
        (*resource).seek(SeekFrom::Start(offset as u64))
    } else if whence == SEEK_CUR {
        (*resource).seek(SeekFrom::Current(offset as i64))
    } else if whence == SEEK_END {
        (*resource).seek(SeekFrom::End(offset as i64))
    } else {
        Err(SysError::new(EINVAL))
    };

    SysError::mux(
        match result {
            Ok(len) => Ok(len as usize),
            Err(err) => Err(err)
        }
    )
}

#[cold]
#[inline(never)]
#[no_mangle]
pub unsafe extern "C" fn _fsync(resource: *mut Resource) -> usize {
    SysError::mux(
        match (*resource).sync() {
            Ok(_) => Ok(0),
            Err(err) => Err(err)
        }
    )
}

#[cold]
#[inline(never)]
#[no_mangle]
pub unsafe extern "C" fn _close(resource: *mut Resource) -> usize {
    drop(Box::from_raw(resource));
    0
}
