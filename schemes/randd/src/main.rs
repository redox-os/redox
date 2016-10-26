#![feature(asm)]

extern crate syscall;

use std::fs::File;
use std::io::{Read, Write};
use std::thread;

use syscall::{Packet, Result, Scheme};

//TODO: Use a CSPRNG, allow write of entropy
struct RandScheme;

impl Scheme for RandScheme {
    fn open(&self, _path: &[u8], _flags: usize, _uid: u32, _gid: u32) -> Result<usize> {
        Ok(0)
    }

    fn dup(&self, file: usize, _buf: &[u8]) -> Result<usize> {
        Ok(file)
    }

    fn read(&self, _file: usize, buf: &mut [u8]) -> Result<usize> {
        let mut i = 0;
        for chunk in buf.chunks_mut(8) {
            let mut rand: u64;
            unsafe {
                asm!("rdrand rax"
                    : "={rax}"(rand)
                    :
                    :
                    : "intel", "volatile");
            }
            for b in chunk.iter_mut() {
                *b = rand as u8;
                rand = rand >> 8;
                i += 1;
            }
        }
        Ok(i)
    }

    fn close(&self, _file: usize) -> Result<usize> {
        Ok(0)
    }
}

fn main(){
    thread::spawn(move || {
        let mut socket = File::create(":rand").expect("rand: failed to create rand scheme");
        let scheme = RandScheme;
        loop {
            let mut packet = Packet::default();
            socket.read(&mut packet).expect("rand: failed to read events from rand scheme");
            scheme.handle(&mut packet);
            socket.write(&packet).expect("rand: failed to write responses to rand scheme");
        }
    });
}
