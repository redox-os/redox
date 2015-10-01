use core::intrinsics::{volatile_load, volatile_store};

pub struct MMIO<T> {
	address: *mut T
}

impl <T> MMIO <T> {
	fn new (address: *mut T) -> MMIO<T> {
		return MMIO {
			address: address
		};
	}

	unsafe fn read(&self) -> T {
		return volatile_load(self.address);
	}

	unsafe fn write(&mut self, value: T) {
	    volatile_store(self.address, value);
	}
}
