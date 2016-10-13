use core::{mem, str};
use core::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT, Ordering};
use spin::{Mutex, Once};

use arch::interrupt::irq::acknowledge;
use context;
use sync::WaitCondition;
use syscall::error::*;
use syscall::flag::EVENT_READ;
use syscall::scheme::Scheme;

pub static IRQ_SCHEME_ID: AtomicUsize = ATOMIC_USIZE_INIT;

/// IRQ queues
static ACKS: Mutex<[usize; 16]> = Mutex::new([0; 16]);
static COUNTS: Mutex<[usize; 16]> = Mutex::new([0; 16]);
static WAITS: Once<[WaitCondition; 16]> = Once::new();

fn init_waits() -> [WaitCondition; 16] {
    [
        WaitCondition::new(), WaitCondition::new(), WaitCondition::new(), WaitCondition::new(),
        WaitCondition::new(), WaitCondition::new(), WaitCondition::new(), WaitCondition::new(),
        WaitCondition::new(), WaitCondition::new(), WaitCondition::new(), WaitCondition::new(),
        WaitCondition::new(), WaitCondition::new(), WaitCondition::new(), WaitCondition::new()
    ]
}

/// Add to the input queue
#[no_mangle]
pub extern fn irq_trigger(irq: u8) {
    COUNTS.lock()[irq as usize] += 1;
    WAITS.call_once(init_waits)[irq as usize].notify();
    context::event::trigger(IRQ_SCHEME_ID.load(Ordering::SeqCst), irq as usize, EVENT_READ, mem::size_of::<usize>());
}

pub struct IrqScheme;

impl Scheme for IrqScheme {
    fn open(&self, path: &[u8], _flags: usize, uid: u32, _gid: u32) -> Result<usize> {
        if uid == 0 {
            let path_str = str::from_utf8(path).or(Err(Error::new(ENOENT)))?;

            let id = path_str.parse::<usize>().or(Err(Error::new(ENOENT)))?;

            if id < COUNTS.lock().len() {
                Ok(id)
            } else {
                Err(Error::new(ENOENT))
            }
        } else {
            Err(Error::new(EACCES))
        }
    }

    fn dup(&self, file: usize) -> Result<usize> {
        Ok(file)
    }

    fn read(&self, file: usize, buffer: &mut [u8]) -> Result<usize> {
        // Ensures that the length of the buffer is larger than the size of a usize
        if buffer.len() >= mem::size_of::<usize>() {
            loop {
                let ack = ACKS.lock()[file];
                let current = COUNTS.lock()[file];
                if ack != current {
                    // Safe if the length of the buffer is larger than the size of a usize
                    assert!(buffer.len() >= mem::size_of::<usize>());
                    unsafe { *(buffer.as_mut_ptr() as *mut usize) = current; }
                    return Ok(mem::size_of::<usize>());
                } else {
                    WAITS.call_once(init_waits)[file].wait();
                }
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

    fn fevent(&self, _file: usize, _flags: usize) -> Result<usize> {
        Ok(0)
    }

    fn fsync(&self, _file: usize) -> Result<usize> {
        Ok(0)
    }

    fn close(&self, _file: usize) -> Result<usize> {
        Ok(0)
    }
}
