use core::marker::PhantomData;
use x86::io;

use super::io::Io;

/// Generic PIO
#[derive(Copy, Clone)]
pub struct Pio<T> {
    port: u16,
    value: PhantomData<T>,
}

impl<T> Pio<T> {
    /// Create a PIO from a given port
    pub const fn new(port: u16) -> Self {
        Pio::<T> {
            port: port,
            value: PhantomData,
        }
    }
}

/// Read/Write for byte PIO
impl Io for Pio<u8> {
    type Value = u8;

    /// Read
    #[inline(always)]
    fn read(&self) -> u8 {
        unsafe { io::inb(self.port) }
    }

    /// Write
    #[inline(always)]
    fn write(&mut self, value: u8) {
        unsafe { io::outb(self.port, value) }
    }
}

/// Read/Write for word PIO
impl Io for Pio<u16> {
    type Value = u16;

    /// Read
    #[inline(always)]
    fn read(&self) -> u16 {
        unsafe { io::inw(self.port) }
    }

    /// Write
    #[inline(always)]
    fn write(&mut self, value: u16) {
        unsafe { io::outw(self.port, value) }
    }
}

/// Read/Write for doubleword PIO
impl Io for Pio<u32> {
    type Value = u32;

    /// Read
    #[inline(always)]
    fn read(&self) -> u32 {
        unsafe { io::inl(self.port) }
    }

    /// Write
    #[inline(always)]
    fn write(&mut self, value: u32) {
        unsafe { io::outl(self.port, value) }
    }
}
