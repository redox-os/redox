use core::intrinsics::{volatile_load, volatile_store};
use core::{u8, u16, u32, u64};

#[repr(packed)]
pub struct Mmio<T> {
    value: T
}

impl <T> Mmio <T> {
    pub fn read(&self) -> T {
        unsafe { volatile_load(&self.value) }
    }

    pub fn write(&mut self, value: T) {
        unsafe { volatile_store(&mut self.value, value) };
    }
}

impl Mmio <u8> {
    pub fn readf(&self, flags: u8) -> bool {
        self.read() & flags == flags
    }

    pub fn writef(&mut self, flags: u8, value: bool) {
        if value {
            let value = self.read() | flags;
            self.write(value);
        } else{
            let value = self.read() & (u8::MAX - flags);
            self.write(value);
        }
    }
}

impl Mmio <u16> {
    pub fn readf(&self, flags: u16) -> bool {
        self.read() & flags == flags
    }

    pub fn writef(&mut self, flags: u16, value: bool) {
        if value {
            let value = self.read() | flags;
            self.write(value);
        } else{
            let value = self.read() & (u16::MAX - flags);
            self.write(value);
        }
    }
}

impl Mmio <u32> {
    pub fn readf(&self, flags: u32) -> bool {
        self.read() & flags == flags
    }

    pub fn writef(&mut self, flags: u32, value: bool) {
        if value {
            let value = self.read() | flags;
            self.write(value);
        } else{
            let value = self.read() & (u32::MAX - flags);
            self.write(value);
        }
    }
}

impl Mmio <u64> {
    pub fn readf(&self, flags: u64) -> bool {
        self.read() & flags == flags
    }

    pub fn writef(&mut self, flags: u64, value: bool) {
        if value {
            let value = self.read() | flags;
            self.write(value);
        } else{
            let value = self.read() & (u64::MAX - flags);
            self.write(value);
        }
    }
}
