use core::{u8, u16, u32};

/// PIO8
#[derive(Copy, Clone)]
pub struct Pio8 {
    port: u16,
}

impl Pio8 {
    /// Create a PIO8 from a given port
    pub fn new(port: u16) -> Self {
        return Pio8 { port: port };
    }

    /// Read
    pub unsafe fn read(&self) -> u8 {
        let value: u8;
        asm!("in $0, $1" : "={al}"(value) : "{dx}"(self.port) : "memory" : "intel", "volatile");
        return value;
    }

    /// Write
    pub unsafe fn write(&mut self, value: u8) {
        asm!("out $1, $0" : : "{al}"(value), "{dx}"(self.port) : "memory" : "intel", "volatile");
    }

    pub unsafe fn readf(&self, flags: u8) -> bool {
        self.read() & flags == flags
    }

    pub unsafe fn writef(&mut self, flags: u8, value: bool) {
        if value {
            let value = self.read() | flags;
            self.write(value);
        } else{
            let value = self.read() & (u8::MAX - flags);
            self.write(value);
        }
    }
}

// TODO: Remove
pub unsafe fn inb(port: u16) -> u8 {
    return Pio8::new(port).read();
}

// TODO: Remove
pub unsafe fn outb(port: u16, value: u8) {
    Pio8::new(port).write(value);
}

/// PIO16
#[derive(Copy, Clone)]
pub struct Pio16 {
    port: u16,
}

impl Pio16 {
    /// Create a new PIO16 from a given port
    pub fn new(port: u16) -> Self {
        return Pio16 { port: port };
    }

    /// Read
    pub unsafe fn read(&self) -> u16 {
        let value: u16;
        asm!("in $0, $1" : "={ax}"(value) : "{dx}"(self.port) : "memory" : "intel", "volatile");
        return value;
    }

    /// Write
    pub unsafe fn write(&mut self, value: u16) {
        asm!("out $1, $0" : : "{ax}"(value), "{dx}"(self.port) : "memory" : "intel", "volatile");
    }

    pub unsafe fn readf(&self, flags: u16) -> bool {
        self.read() & flags == flags
    }

    pub unsafe fn writef(&mut self, flags: u16, value: bool) {
        if value {
            let value = self.read() | flags;
            self.write(value);
        } else{
            let value = self.read() & (u16::MAX - flags);
            self.write(value);
        }
    }
}

// TODO: Remove
pub unsafe fn inw(port: u16) -> u16 {
    return Pio16::new(port).read();
}

// TODO: Remove
pub unsafe fn outw(port: u16, value: u16) {
    Pio16::new(port).write(value);
}

/// PIO32
#[derive(Copy, Clone)]
pub struct Pio32 {
    port: u16,
}

impl Pio32 {
    /// Create a new PIO32 from a port
    pub fn new(port: u16) -> Self {
        return Pio32 { port: port };
    }

    /// Read
    pub unsafe fn read(&self) -> u32 {
        let value: u32;
        asm!("in $0, $1" : "={eax}"(value) : "{dx}"(self.port) : "memory" : "intel", "volatile");
        return value;
    }

    /// Write
    pub unsafe fn write(&mut self, value: u32) {
        asm!("out $1, $0" : : "{eax}"(value), "{dx}"(self.port) : "memory" : "intel", "volatile");
    }

    pub unsafe fn readf(&self, flags: u32) -> bool {
        self.read() & flags == flags
    }

    pub unsafe fn writef(&mut self, flags: u32, value: bool) {
        if value {
            let value = self.read() | flags;
            self.write(value);
        } else{
            let value = self.read() & (u32::MAX - flags);
            self.write(value);
        }
    }
}

// TODO: Remove
pub unsafe fn ind(port: u16) -> u32 {
    return Pio32::new(port).read();
}

// TODO: Remove
pub unsafe fn outd(port: u16, value: u32) {
    Pio32::new(port).write(value);
}
