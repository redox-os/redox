extern crate syscall;

use std::fs::File;
use std::io::{Read, Write};
use std::str;

use syscall::{Packet, Result, Scheme};

struct ExampleScheme;

impl Scheme for ExampleScheme {
    fn open(&self, path: &[u8], _flags: usize, uid: u32, gid: u32) -> Result<usize> {
        println!("{} from {}:{}", unsafe { str::from_utf8_unchecked(path) }, uid, gid);
        Ok(0)
    }

    fn dup(&self, file: usize, _buf: &[u8]) -> Result<usize> {
        Ok(file)
    }

    fn close(&self, _file: usize) -> Result<usize> {
        Ok(0)
    }
}

fn main(){
    // Daemonize
    if unsafe { syscall::clone(0).unwrap() } == 0 {
        let mut socket = File::create(":example").expect("example: failed to create example scheme");
        let scheme = ExampleScheme;
        loop {
            let mut packet = Packet::default();
            socket.read(&mut packet).expect("example: failed to read events from example scheme");
            println!("{:?}", packet);
            scheme.handle(&mut packet);
            socket.write(&packet).expect("example: failed to write responses to example scheme");
        }
    }
}
