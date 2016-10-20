#![feature(rand)]

extern crate netutils;
extern crate resource_scheme;
extern crate syscall;

use std::fs::File;
use std::io::{Read, Write};
use std::thread;

use resource_scheme::ResourceScheme;
use syscall::Packet;

use scheme::TcpScheme;

mod resource;
mod scheme;

fn main() {
    thread::spawn(move || {
        let mut socket = File::create(":tcp").expect("tcpd: failed to create tcp scheme");
        let scheme = TcpScheme;
        loop {
            let mut packet = Packet::default();
            socket.read(&mut packet).expect("tcpd: failed to read events from tcp scheme");
            scheme.handle(&mut packet);
            socket.write(&packet).expect("tcpd: failed to write responses to tcp scheme");
        }
    });
}
