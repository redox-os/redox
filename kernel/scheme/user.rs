use alloc::arc::Weak;
use collections::{BTreeMap, VecDeque};
use core::sync::atomic::{AtomicUsize, Ordering};
use core::{mem, usize};
use spin::Mutex;

use context;
use syscall::{convert_to_result, Call, Error, Result};

use super::Scheme;

#[derive(Copy, Clone, Debug)]
#[repr(packed)]
pub struct Packet {
    pub id: usize,
    pub a: usize,
    pub b: usize,
    pub c: usize,
    pub d: usize
}

pub struct UserInner {
    next_id: AtomicUsize,
    todo: Mutex<VecDeque<Packet>>,
    done: Mutex<BTreeMap<usize, usize>>
}

impl UserInner {
    pub fn new() -> UserInner {
        UserInner {
            next_id: AtomicUsize::new(0),
            todo: Mutex::new(VecDeque::new()),
            done: Mutex::new(BTreeMap::new())
        }
    }

    pub fn call(&self, a: Call, b: usize, c: usize, d: usize) -> Result<usize> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);

        let packet = Packet {
            id: id,
            a: a as usize,
            b: b,
            c: c,
            d: d
        };

        self.todo.lock().push_back(packet);

        loop {
            if let Some(a) = self.done.lock().remove(&id) {
                return convert_to_result(a);
            }

            unsafe { context::switch(); }
        }
    }

    pub fn read(&self, buf: &mut [u8]) -> Result<usize> {
        let packet_size = mem::size_of::<Packet>();
        let len = buf.len()/packet_size;
        if len > 0 {
            loop {
                let mut i = 0;
                {
                    let mut todo = self.todo.lock();
                    while ! todo.is_empty() && i < len {
                        let packet = todo.pop_front().unwrap();
                        unsafe { *(buf.as_mut_ptr() as *mut Packet).offset(i as isize) = packet; }
                        i += 1;
                    }
                }

                if i > 0 {
                    return Ok(i * packet_size);
                } else {
                    unsafe { context::switch(); }
                }
            }
        } else {
            Ok(0)
        }
    }

    pub fn write(&self, buf: &[u8]) -> Result<usize> {
        let packet_size = mem::size_of::<Packet>();
        let len = buf.len()/packet_size;
        let mut i = 0;
        while i < len {
            let packet = unsafe { *(buf.as_ptr() as *const Packet).offset(i as isize) };
            self.done.lock().insert(packet.id, packet.a);

            i += 1;
        }

        Ok(i * packet_size)
    }
}

/// UserInner has to be wrapped
pub struct UserScheme {
    inner: Weak<UserInner>
}

impl UserScheme {
    pub fn new(inner: Weak<UserInner>) -> UserScheme {
        UserScheme {
            inner: inner
        }
    }
}

impl Scheme for UserScheme {
    fn open(&self, path: &[u8], flags: usize) -> Result<usize> {
        let inner = self.inner.upgrade().ok_or(Error::NoDevice)?;
        inner.call(Call::Open, path.as_ptr() as usize, path.len(), flags)
    }

    fn dup(&self, file: usize) -> Result<usize> {
        let inner = self.inner.upgrade().ok_or(Error::NoDevice)?;
        inner.call(Call::Dup, file, 0, 0)
    }

    fn read(&self, file: usize, buf: &mut [u8]) -> Result<usize> {
        let inner = self.inner.upgrade().ok_or(Error::NoDevice)?;
        inner.call(Call::Read, file, buf.as_mut_ptr() as usize, buf.len())
    }

    fn write(&self, file: usize, buf: &[u8]) -> Result<usize> {
        let inner = self.inner.upgrade().ok_or(Error::NoDevice)?;
        inner.call(Call::Write, file, buf.as_ptr() as usize, buf.len())
    }

    fn fsync(&self, file: usize) -> Result<()> {
        let inner = self.inner.upgrade().ok_or(Error::NoDevice)?;
        inner.call(Call::FSync, file, 0, 0).and(Ok(()))
    }

    fn close(&self, file: usize) -> Result<()> {
        let inner = self.inner.upgrade().ok_or(Error::NoDevice)?;
        inner.call(Call::Close, file, 0, 0).and(Ok(()))
    }
}
