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


pub struct URL<'a> {
    pub scheme: &'a str,
    pub host: &'a str,
    pub port: &'a str,
    pub path: &'a str
}

impl<'a> URL<'a> {
    pub fn d(&self){
        d(self.scheme);
        d("://");
        d(self.host);
        d(":");
        d(self.port);
        d("/");
        d(self.path);
        dl();
    }
}

const TEST: &'static str = "Test string from user application!\n";

// Register to handle port_io://*
#[no_mangle]
pub fn main(){
    d(TEST);

    register_url_scheme("port_io");
}

pub fn register_url_scheme(scheme: &str){
    d("Registering scheme: ");
    d(scheme);
    dl();
}
