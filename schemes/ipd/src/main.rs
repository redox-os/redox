#![feature(rand)]

//! Implementation of the IP Scheme as a userland driver.
//!
//! # Role
//!
//! See https://en.wikipedia.org/wiki/Internet_Protocol for more details about the
//! IP protocol. Clients will often prefer using either higher-level protocols TCP
//! or UDP, both of which are built upon IP.
//!
//! # URL Syntax
//!
//! To open a IP connection, use `ip:[host]/protocol`.
//!
//! * If `host` is specified, it must be an ipv4 number (e.g. `192.168.0.1`)
//! and the connection may be used immediately to send/receive data.
//! * If `host` is omitted, this connectino will wait for a distant peer to
//! connect.
//! * The `protocol` is the hex-based number of the ip protocol
//! (see http://www.iana.org/assignments/protocol-numbers/protocol-numbers.xhtml).

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
