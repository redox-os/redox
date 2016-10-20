extern crate netutils;
extern crate resource_scheme;
extern crate syscall;

use std::fs::File;
use std::io::{Read, Write};
use std::thread;

use resource_scheme::ResourceScheme;
use syscall::Packet;

use scheme::EthernetScheme;

mod resource;
mod scheme;

fn main() {
    thread::spawn(move || {
        let mut socket = File::create(":ethernet").expect("ethernetd: failed to create ethernet scheme");
        let scheme = EthernetScheme;
        loop {
            let mut packet = Packet::default();
            socket.read(&mut packet).expect("ethernetd: failed to read events from ethernet scheme");
            scheme.handle(&mut packet);
            socket.write(&packet).expect("ethernetd: failed to write responses to ethernet scheme");
        }
    });
}
