#![feature(asm)]
#![feature(core)]
#![feature(no_std)]
#![no_std]

extern crate core;

use common::debug::*;

mod common {
    pub mod debug;
	pub mod pio;
}

const TEST: &'static str = "Test string from user application!\n";

#[no_mangle]
pub fn main() {
    d(TEST);
}

pub struct URL<'a> {
    pub scheme: &'a str,
    pub host: &'a str,
    pub port: &'a str,
    pub path: &'a str
}

pub fn str_to_int(string: &str, radix: isize) -> isize{
    return 0;
}

pub fn register_url_scheme(scheme: &str){
    d("Registering scheme: ");
    d(scheme);
    dl();
}

// Register to handle port_io://*
#[no_mangle]
pub fn module(){
    register_url_scheme("port_io");
}

// Open a matching URL, do not open if the port is not available.
#[no_mangle]
pub fn open(url: URL) -> isize {
    let port = str_to_int(url.path, 16);
    if port >= 0 && port < 65536 {
        return port;
    }else{
        //Somehow handle error
        return -1;
    }
}

// Read data from a port.
// The OS handles the resource key, making sure permissions are kept (the user cannot make up a resource number)
#[no_mangle]
pub unsafe fn read(port: isize, data: &mut [u8]) -> usize {
    let mut count = -1;

    if port >= 0 && port < 65536 {
        count = 0;
        for byte in data {
            asm!("in $0, $1\n" : "={al}"(*byte) : "{dx}"(port as u16) : : "intel");
        }
    }

    count
}

// Write data to a port.
#[no_mangle]
pub unsafe fn write(port: isize, data: &[u8]) -> usize {
    let mut count = -1;

    if port >= 0 && port < 65536 {
        count = 0;
        for byte in data {
            asm!("out $0, $1\n" : : "{dx}"(port as u16), "{al}"(*byte) : : "intel");
            count += 1;
        }
    }

    count
}

// Close a port.
// This operation will always be successful as opening the port did not create any garbage.
#[no_mangle]
pub fn close(port: isize) -> bool {
    return true;
}