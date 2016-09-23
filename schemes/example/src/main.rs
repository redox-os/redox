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
        let events = syscall::open("event:", 0).unwrap();

        let a = syscall::open("display:", 0).unwrap();
        syscall::fevent(a, syscall::EVENT_READ).unwrap();

        loop {
            let mut event = syscall::Event::default();
            syscall::read(events, &mut event).unwrap();
            println!("{:?}", event);
        }

        let _ = syscall::close(events);
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
