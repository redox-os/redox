use std::fs::File;
use std::io::{Read, Write};

use system::error::{Error, Result, ENOENT, EBADF};
use system::scheme::{Packet, Scheme};

extern crate system;

struct ExampleScheme;

impl Scheme for ExampleScheme {
    fn open(&mut self, path: &str, flags: usize, mode: usize) -> Result {
        println!("open {:X} = {}, {:X}, {:X}", path.as_ptr() as usize, path, flags, mode);
        Ok(0)
    }

    #[allow(unused_variables)]
    fn unlink(&mut self, path: &str) -> Result {
        println!("unlink {}", path);
        Err(Error::new(ENOENT))
    }

    #[allow(unused_variables)]
    fn mkdir(&mut self, path: &str, mode: usize) -> Result {
        println!("mkdir {}, {:X}", path, mode);
        Err(Error::new(ENOENT))
    }

    /* Resource operations */

    #[allow(unused_variables)]
    fn read(&mut self, id: usize, buf: &mut [u8]) -> Result {
        println!("read {}, {:X}, {}", id, buf.as_mut_ptr() as usize, buf.len());
        Err(Error::new(EBADF))
    }

    #[allow(unused_variables)]
    fn write(&mut self, id: usize, buf: &[u8]) -> Result {
        println!("write {}, {:X}, {}", id, buf.as_ptr() as usize, buf.len());
        Err(Error::new(EBADF))
    }

    #[allow(unused_variables)]
    fn seek(&mut self, id: usize, pos: usize, whence: usize) -> Result {
        println!("seek {}, {}, {}", id, pos, whence);
        Err(Error::new(EBADF))
    }

    #[allow(unused_variables)]
    fn sync(&mut self, id: usize) -> Result {
        println!("sync {}", id);
        Err(Error::new(EBADF))
    }

    #[allow(unused_variables)]
    fn truncate(&mut self, id: usize, len: usize) -> Result {
        println!("truncate {}, {}", id, len);
        Err(Error::new(EBADF))
    }
}

fn main() {
   //In order to handle example:, we create :example
   let mut scheme = ExampleScheme;
   let mut socket = File::create(":example").unwrap();
   loop {
       let mut packet = Packet::default();
       if socket.read(&mut packet).unwrap() == 0 {
           panic!("Unexpected EOF");
       }
       println!("Recv {:?}", packet);

       scheme.handle(&mut packet);

       socket.write(&packet).unwrap();
       println!("Sent {:?}", packet);
   }
}
