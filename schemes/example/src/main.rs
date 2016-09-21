extern crate syscall;

use std::fs::File;
use std::io::{Read, Write};
use std::str;
use std::thread;

use syscall::{Packet, Result, Scheme};

struct ExampleScheme;

impl Scheme for ExampleScheme {
    fn open(&self, path: &[u8], _flags: usize) -> Result<usize> {
        println!("{}", unsafe { str::from_utf8_unchecked(path) });
        Ok(0)
    }

    fn dup(&self, file: usize) -> Result<usize> {
        Ok(file)
    }

    fn close(&self, _file: usize) -> Result<usize> {
        Ok(0)
    }
}

fn main(){
    {
        let fd = syscall::open("display:", 0).expect("example: failed to open display");
        let mut buf = *b"          \n";
        let mut i: usize = 0;
        while i < 10000000000 {
            i += 1;
            let mut n = i;
            let mut e = 10;
            while e > 0 && n > 0 {
                e -= 1;
                buf[e] = (n % 10) as u8 + b'0';
                n = n / 10;
            }
            syscall::write(fd, &buf[e..]);
        }
        syscall::close(fd);
    }

    thread::spawn(move || {
        let mut socket = File::create(":example").expect("example: failed to create example scheme");
        let scheme = ExampleScheme;
        loop {
            let mut packet = Packet::default();
            socket.read(&mut packet).expect("example: failed to read events from example scheme");
            println!("{:?}", packet);
            scheme.handle(&mut packet);
            socket.write(&packet).expect("example: failed to write responses to example scheme");
        }
    });
}
