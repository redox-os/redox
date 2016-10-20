#![feature(rand)]

extern crate netutils;
extern crate resource_scheme;
extern crate syscall;

use std::fs::File;
use std::io::{Read, Write};
use std::thread;

use resource_scheme::ResourceScheme;
use syscall::Packet;

use scheme::IpScheme;

mod resource;
mod scheme;

fn main() {
    thread::spawn(move || {
        let mut socket = File::create(":ip").expect("ipd: failed to create ip scheme");
        let scheme = IpScheme::new();
        loop {
            let mut packet = Packet::default();
            socket.read(&mut packet).expect("ipd: failed to read events from ip scheme");
            scheme.handle(&mut packet);
            socket.write(&packet).expect("ipd: failed to write responses to ip scheme");
        }
    });
}
