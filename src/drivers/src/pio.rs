/// PIO8
#[derive(Copy, Clone)]
pub struct PIO8 {
    port: u16,
}

impl PIO8 {
    /// Create a PIO8 from a given port
    pub fn new(port: u16) -> Self {
        return PIO8 { port: port };
    }

    /// Read
    pub unsafe fn read(&self) -> u8 {
        let value: u8;
        asm!("in $0, $1" : "={al}"(value) : "{dx}"(self.port) : : "intel", "volatile");
        return value;
    }

    /// Write
    pub unsafe fn write(&mut self, value: u8) {
        asm!("out $1, $0" : : "{al}"(value), "{dx}"(self.port) : : "intel", "volatile");
    }
}

//TODO: Remove
pub unsafe fn inb(port: u16) -> u8 {
    return PIO8::new(port).read();
}

//TODO: Remove
pub unsafe fn outb(port: u16, value: u8) {
    PIO8::new(port).write(value);
}

/// PIO16
#[derive(Copy, Clone)]
pub struct PIO16 {
    port: u16,
}

impl PIO16 {
    /// Create a new PIO16 from a given port
    pub fn new(port: u16) -> Self {
        return PIO16 { port: port };
    }

    /// Read
    pub unsafe fn read(&self) -> u16 {
        let value: u16;
        asm!("in $0, $1" : "={ax}"(value) : "{dx}"(self.port) : : "intel", "volatile");
        return value;
    }

    /// Write
    pub unsafe fn write(&mut self, value: u16) {
        asm!("out $1, $0" : : "{ax}"(value), "{dx}"(self.port) : : "intel", "volatile");
    }
}

//TODO: Remove
pub unsafe fn inw(port: u16) -> u16 {
    return PIO16::new(port).read();
}

//TODO: Remove
pub unsafe fn outw(port: u16, value: u16) {
    PIO16::new(port).write(value);
}

/// PIO32
#[derive(Copy, Clone)]
pub struct PIO32 {
    port: u16,
}

impl PIO32 {
    /// Create a new PIO32 from a port
    pub fn new(port: u16) -> Self {
        return PIO32 { port: port };
    }

    /// Read
    pub unsafe fn read(&self) -> u32 {
        let value: u32;
        asm!("in $0, $1" : "={eax}"(value) : "{dx}"(self.port) : : "intel", "volatile");
        return value;
    }

    /// Write
    pub unsafe fn write(&mut self, value: u32) {
        asm!("out $1, $0" : : "{eax}"(value), "{dx}"(self.port) : : "intel", "volatile");
    }
}

//TODO: Remove
pub unsafe fn ind(port: u16) -> u32 {
    return PIO32::new(port).read();
}

//TODO: Remove
pub unsafe fn outd(port: u16, value: u32) {
    PIO32::new(port).write(value);
}
