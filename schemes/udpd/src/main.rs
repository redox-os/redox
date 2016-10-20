#![feature(rand)]

extern crate netutils;
extern crate resource_scheme;
extern crate syscall;

use std::fs::File;
use std::io::{Read, Write};
use std::thread;

use resource_scheme::ResourceScheme;
use syscall::Packet;

use scheme::UdpScheme;

mod resource;
mod scheme;

fn main() {
    thread::spawn(move || {
        let mut socket = File::create(":udp").expect("udpd: failed to create udp scheme");
        let scheme = UdpScheme;
        loop {
            let mut packet = Packet::default();
            socket.read(&mut packet).expect("udpd: failed to read events from udp scheme");
            scheme.handle(&mut packet);
            socket.write(&packet).expect("udpd: failed to write responses to udp scheme");
        }
    });
}
