use core::{mem, str};

use arch::interrupt::irq::{ACKS, COUNTS, acknowledge};
use syscall::error::*;
use syscall::scheme::Scheme;

pub struct IrqScheme;

impl Scheme for IrqScheme {
    fn open(&self, path: &[u8], _flags: usize, _uid: u32, _gid: u32) -> Result<usize> {
        let path_str = str::from_utf8(path).or(Err(Error::new(ENOENT)))?;

        let id = path_str.parse::<usize>().or(Err(Error::new(ENOENT)))?;

        if id < COUNTS.lock().len() {
            Ok(id)
        } else {
            Err(Error::new(ENOENT))
        }
    }

    fn dup(&self, file: usize) -> Result<usize> {
        Ok(file)
    }

    fn read(&self, file: usize, buffer: &mut [u8]) -> Result<usize> {
        // Ensures that the length of the buffer is larger than the size of a usize
        if buffer.len() >= mem::size_of::<usize>() {
            let ack = ACKS.lock()[file];
            let current = COUNTS.lock()[file];
            if ack != current {
                // Safe if the length of the buffer is larger than the size of a usize
                assert!(buffer.len() >= mem::size_of::<usize>());
                unsafe { *(buffer.as_mut_ptr() as *mut usize) = current; }
                Ok(mem::size_of::<usize>())
            } else {
                Ok(0)
            }
        } else {
            Err(Error::new(EINVAL))
        }
    }

    fn write(&self, file: usize, buffer: &[u8]) -> Result<usize> {
        if buffer.len() >= mem::size_of::<usize>() {
            assert!(buffer.len() >= mem::size_of::<usize>());
            let ack = unsafe { *(buffer.as_ptr() as *const usize) };
            let current = COUNTS.lock()[file];
            if ack == current {
                ACKS.lock()[file] = ack;
                unsafe { acknowledge(file); }
                Ok(mem::size_of::<usize>())
            } else {
                Ok(0)
            }
        } else {
            Err(Error::new(EINVAL))
        }
    }

    fn fsync(&self, _file: usize) -> Result<usize> {
        Ok(0)
    }

    fn close(&self, _file: usize) -> Result<usize> {
        Ok(0)
    }
}
