#![feature(asm)]
#![feature(rand)]

extern crate syscall;
extern crate raw_cpuid;
extern crate rand;

use std::fs::File;
use std::io::{Read, Write};

use rand::chacha::ChaChaRng;
use rand::Rng;

use raw_cpuid::CpuId;

use syscall::{Packet, Result, SchemeMut};

//TODO: Use a CSPRNG, allow write of entropy
struct RandScheme {
   prng: ChaChaRng
}

impl SchemeMut for RandScheme {
    fn open(&mut self, _path: &[u8], _flags: usize, _uid: u32, _gid: u32) -> Result<usize> {
        Ok(0)
    }

    fn dup(&mut self, file: usize, _buf: &[u8]) -> Result<usize> {
        Ok(file)
    }

    fn read(&mut self, _file: usize, buf: &mut [u8]) -> Result<usize> {
        let mut i = 0;
        for chunk in buf.chunks_mut(8) {
            let mut rand = self.prng.next_u64();
            for b in chunk.iter_mut() {
                *b = rand as u8;
                rand = rand >> 8;
                i += 1;
            }
        }
        Ok(i)
    }

    fn close(&mut self, _file: usize) -> Result<usize> {
        Ok(0)
    }
}

fn main(){
    let has_rdrand = CpuId::new().get_feature_info().unwrap().has_rdrand();

    // Daemonize
    if unsafe { syscall::clone(0).unwrap() } == 0 {
        let mut socket = File::create(":rand").expect("rand: failed to create rand scheme");

        let mut rng = ChaChaRng::new_unseeded();

        if has_rdrand {
            println!("rand: seeding with rdrand");
            let rand: u64;
            unsafe {
                asm!("rdrand rax"
                    : "={rax}"(rand)
                    :
                    :
                    : "intel", "volatile");
            }
            rng.set_counter(0, rand);
        } else {
            println!("rand: unseeded");
        }

        let mut scheme = RandScheme{prng: rng};
        loop {
            let mut packet = Packet::default();
            socket.read(&mut packet).expect("rand: failed to read events from rand scheme");
            scheme.handle(&mut packet);
            socket.write(&packet).expect("rand: failed to write responses to rand scheme");
        }
    }
}
