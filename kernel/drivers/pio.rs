use core::cmp::PartialEq;
use core::ops::{BitAnd, BitOr, Not};
use core::marker::PhantomData;

pub trait ReadWrite<T>
{
    fn read(&self) -> T;
    fn write(&self, value: T);
}

/// Generic PIO
#[derive(Copy, Clone)]
pub struct Pio<T> {
    port: u16,
    value: PhantomData<T>,
}

/// Read/Write for byte PIO
impl ReadWrite<u8> for Pio<u8> {
    /// Read
    fn read(&self) -> u8 {
        let value: u8;
        unsafe {
            asm!("in $0, $1" : "={al}"(value) : "{dx}"(self.port) : "memory" : "intel", "volatile");
        }
        value
    }

    /// Write
    fn write(&self, value: u8) {
        unsafe {
            asm!("out $1, $0" : : "{al}"(value), "{dx}"(self.port) : "memory" : "intel", "volatile");
        }
    }
}

/// Read/Write for word PIO
impl ReadWrite<u16> for Pio<u16> {
    /// Read
    fn read(&self) -> u16 {
        let value: u16;
        unsafe {
            asm!("in $0, $1" : "={ax}"(value) : "{dx}"(self.port) : "memory" : "intel", "volatile");
        }
        value
    }

    /// Write
    fn write(&self, value: u16) {
        unsafe {
            asm!("out $1, $0" : : "{ax}"(value), "{dx}"(self.port) : "memory" : "intel", "volatile");
        }
    }
}

/// Read/Write for doubleword PIO
impl ReadWrite<u32> for Pio<u32> {
    /// Read
    fn read(&self) -> u32 {
        let value: u32;
        unsafe {
            asm!("in $0, $1" : "={eax}"(value) : "{dx}"(self.port) : "memory" : "intel", "volatile");
        }
        value
    }

    /// Write
    fn write(&self, value: u32) {
        unsafe {
            asm!("out $1, $0" : : "{eax}"(value), "{dx}"(self.port) : "memory" : "intel", "volatile");
        }
    }
}

impl<T> Pio<T>
    where Pio<T>: ReadWrite<T>,
          T: BitAnd<Output = T> + BitOr<Output = T> + PartialEq<T> + Not<Output = T> + Copy
{
    /// Create a PIO from a given port
    pub fn new(port: u16) -> Self {
        Pio::<T> {
            port: port,
            value: PhantomData,
        }
    }

    pub fn readf(&self, flags: T) -> bool {
        (self.read() & flags) as T == flags
    }

    pub fn writef(&mut self, flags: T, value: bool) {
        let tmp: T = match value {
            true => self.read() | flags,
            false => self.read() & !flags,
        };
        self.write(tmp);
    }
}

// TODO: Remove
pub unsafe fn inb(port: u16) -> u8 {
    Pio::<u8>::new(port).read()
}

// TODO: Remove
pub unsafe fn outb(port: u16, value: u8) {
    Pio::<u8>::new(port).write(value);
}

// TODO: Remove
pub unsafe fn inw(port: u16) -> u16 {
    Pio::<u16>::new(port).read()
}

// TODO: Remove
pub unsafe fn outw(port: u16, value: u16) {
    Pio::<u16>::new(port).write(value);
}

// TODO: Remove
pub unsafe fn ind(port: u16) -> u32 {
    Pio::<u32>::new(port).read()
}

// TODO: Remove
pub unsafe fn outd(port: u16, value: u32) {
    Pio::<u32>::new(port).write(value);
}
