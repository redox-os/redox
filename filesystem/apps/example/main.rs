use std::fs::File;
use std::io::{Read, Write};

use system::scheme::{Packet, Scheme};

extern crate system;

struct ExampleScheme;

impl Scheme for ExampleScheme {

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
