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
//! and the connection may be used immediately to send/receive data. Ip v4 number
//! `127.0.0.1` is hardwired to the loopback device (i.e. localhost), which doesn't
//! access any physical device and in which data can only be read by the same
//! connection that has written it.
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

#[cfg(test)]
fn test() {
    use scheme::IpScheme;

    println!("* Test that we can read a simple packet from the same connection.");
    let bytes = "TEST".as_bytes();
    let mut scheme = IpScheme::new();
    let a = scheme.open(&"ip:127.0.0.1/11".as_bytes(), 0, 0, 0).unwrap();
    let num_bytes_written = scheme.write(a, bytes).unwrap();
    assert_eq!(num_bytes_written, bytes.len());

    let mut buf = [0;65536];
    let num_bytes_read = scheme.read(a, &mut buf).unwrap();
    assert_eq!(num_bytes_read, num_bytes_written);

    let bytes_read = &buf[0..num_bytes_read];
    assert_eq!(bytes, bytes_read);



    println!("* Test that the loopback is now empty.");
    let num_bytes_read = scheme.read(a, &mut buf).unwrap();
    assert_eq!(num_bytes_read, 0);



    println!("* Test that we can read the same packet from a different connection.");
    let num_bytes_written = scheme.write(a, bytes).unwrap();
    assert_eq!(num_bytes_written, bytes.len());

    let b = scheme.open("ip:127.0.0.1/11".as_bytes(), 0, 0, 0).unwrap();

    let num_bytes_read = scheme.read(b, &mut buf).unwrap();
    assert_eq!(num_bytes_read, num_bytes_written);

    let bytes_read = &buf[0..num_bytes_read];
    assert_eq!(bytes, bytes_read);



    println!("* Test that the loopback is now empty for both connections.");

    let num_bytes_read = scheme.read(a, &mut buf).unwrap();
    assert_eq!(num_bytes_read, 0);

    let num_bytes_read = scheme.read(b, &mut buf).unwrap();
    assert_eq!(num_bytes_read, 0);




    println!("* Push a number of packets, check that we get them in the right order.");
    let mut payloads : Vec<String> = (0..100).map(|i| format!("TEST {}", i)).collect();
    for payload in &payloads {
        let bytes = payload.into_bytes();
        let num_bytes_written = scheme.write(a, &bytes).unwrap();
        assert_eq!(bytes.len(), num_bytes_written);
    }
    for payload in &payloads {
        let bytes = payload.into_bytes();
        let mut buf = [0;65536];
        let num_bytes_read = scheme.read(a, &mut buf).unwrap();
        assert_eq!(bytes.len(), num_bytes_read);
        let bytes_read = &buf[0..num_bytes_read];
        assert_eq!(bytes, bytes_read);
    }
}