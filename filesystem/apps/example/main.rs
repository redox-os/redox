use std::fs::File;
use std::io::{Read, Write};

use std::ops::{Deref, DerefMut};

#[derive(Copy, Clone, Debug, Default)]
#[repr(packed)]
struct Packet {
    id: i64,
    a: i64,
    b: i64,
    c: i64,
    d: i64
}

impl Deref for Packet {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(self as *const Packet as *const u8, std::mem::size_of::<Packet>()) as &[u8]
        }
    }
}

impl DerefMut for Packet {
    fn deref_mut(&mut self) -> &mut [u8] {
        unsafe {
            std::slice::from_raw_parts_mut(self as *mut Packet as *mut u8, std::mem::size_of::<Packet>()) as &mut [u8]
        }
    }
}

fn main() {
   //In order to handle example:, we create :example
   let mut scheme = File::create(":example").unwrap();
   loop {
       let mut packet = Packet::default();
       if scheme.read(&mut packet).unwrap() == 0 {
           panic!("Unexpected EOF");
       }

       println!("Received: {:?}", packet);

       packet.a = 0;
       scheme.write(&packet).unwrap();
   }
}
