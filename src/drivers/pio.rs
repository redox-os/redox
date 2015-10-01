pub struct PIO8 {
	port: u16
}

impl PIO8 {
	fn new (port: u16) -> PIO8 {
		return PIO8 {
			port: port
		};
	}

	unsafe fn read(&self) -> u8 {
		let value: u8;
	    asm!("in $0, $1" : "={al}"(value) : "{dx}"(port) : : "intel", "volatile");
		return value;
	}

	unsafe fn write(&mut self, value: u8) {
	    asm!("out $1, $0" : : "{al}"(value), "{dx}"(port) : : "intel", "volatile");
	}
}

pub struct PIO16 {
	port: u16
}

impl PIO16 {
	fn new (port: u16) -> PIO16 {
		return PIO16 {
			port: port
		};
	}

	unsafe fn read(&self) -> u16 {
		let value: u16;
	    asm!("in $0, $1" : "={ax}"(value) : "{dx}"(port) : : "intel", "volatile");
		return value;
	}

	unsafe fn write(&mut self, value: u16) {
	    asm!("out $1, $0" : : "{ax}"(value), "{dx}"(port) : : "intel", "volatile");
	}
}

pub struct PIO32 {
	port: u16
}

impl PIO32 {
	fn new (port: u16) -> PIO32 {
		return PIO32 {
			port: port
		};
	}

	unsafe fn read(&self) -> u32 {
		let value: u32;
	    asm!("in $0, $1" : "={eax}"(value) : "{dx}"(port) : : "intel", "volatile");
		return value;
	}

	unsafe fn write(&mut self, value: u32) {
	    asm!("out $1, $0" : : "{eax}"(value), "{dx}"(port) : : "intel", "volatile");
	}
}
