#![crate_name="io"]
#![crate_type="lib"]
#![feature(asm)]
#![feature(core_intrinsics)]
#![no_std]

use core::cmp::PartialEq;
use core::ops::{BitAnd, BitOr, Not};

pub use self::mmio::*;
pub use self::pio::*;

mod mmio;
mod pio;

pub trait Io<T> {
    fn read(&self) -> T;
    fn write(&mut self, value: T);

    fn readf(&self, flags: T) -> bool where T: BitAnd<Output = T> + PartialEq<T> + Copy {
        (self.read() & flags) as T == flags
    }

    fn writef(&mut self, flags: T, value: bool) where T: BitAnd<Output = T> + BitOr<Output = T> + Not<Output = T> {
        let tmp: T = match value {
            true => self.read() | flags,
            false => self.read() & !flags,
        };
        self.write(tmp);
    }
}
