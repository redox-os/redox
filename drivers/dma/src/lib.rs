#![feature(question_mark)]

extern crate syscall;

use std::{mem, ptr};
use std::ops::{Deref, DerefMut};

use syscall::Result;

struct PhysBox {
    address: usize,
    size: usize
}

impl PhysBox {
    fn new(size: usize) -> Result<PhysBox> {
        let address = unsafe { syscall::physalloc(size)? };
        Ok(PhysBox {
            address: address,
            size: size
        })
    }
}

impl Drop for PhysBox {
    fn drop(&mut self) {
        let _ = unsafe { syscall::physfree(self.address, self.size) };
    }
}

pub struct Dma<T> {
    phys: PhysBox,
    virt: *mut T
}

impl<T> Dma<T> {
    pub fn new(value: T) -> Result<Dma<T>> {
        let phys = PhysBox::new(mem::size_of::<T>())?;
        let virt = unsafe { syscall::physmap(phys.address, phys.size, syscall::MAP_WRITE)? } as *mut T;
        unsafe { ptr::write(virt, value); }
        Ok(Dma {
            phys: phys,
            virt: virt
        })
    }

    pub fn zeroed() -> Result<Dma<T>> {
        let phys = PhysBox::new(mem::size_of::<T>())?;
        let virt = unsafe { syscall::physmap(phys.address, phys.size, syscall::MAP_WRITE)? } as *mut T;
        unsafe { ptr::write_bytes(virt as *mut u8, 0, phys.size); }
        Ok(Dma {
            phys: phys,
            virt: virt
        })
    }

    pub fn physical(&self) -> usize {
        self.phys.address
    }
}

impl<T> Deref for Dma<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.virt }
    }
}

impl<T> DerefMut for Dma<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.virt }
    }
}

impl<T> Drop for Dma<T> {
    fn drop(&mut self) {
        unsafe { drop(ptr::read(self.virt)); }
        let _ = unsafe { syscall::physunmap(self.virt as usize) };
    }
}
