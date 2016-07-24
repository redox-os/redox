use std::fs::File;
use std::io::{Read, Write};
use std::{mem, slice};

use dns::Dns;

mod dns;

pub fn htons(u: u16) -> u16 {
    u.to_be()
}

pub fn ntohs(u: u16) -> u16 {
    use std::u16;
    u16::from_be(u)
}

fn main(){
    let req = Dns {
        transaction_id: htons(0xBEEF),
        flags: htons(0x0100),
        questions: htons(1),
        answers: htons(0),
        authorities: htons(0),
        additional: htons(0),
        req: *b"\x06static\x05redox\x03org\0",
        req_type: htons(0x0001),
        req_class: htons(0x0001),
    };

    let mut socket = File::open("udp:10.0.2.3:53").unwrap();
    let sent = socket.write(unsafe { slice::from_raw_parts(&req as *const Dns as *const u8, mem::size_of::<Dns>()) }).unwrap();

    socket.flush().unwrap();

    let mut buf = [0; 65536];
    let count = socket.read(&mut buf).unwrap();
}
