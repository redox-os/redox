use collections::{BTreeMap, VecDeque};
use core::sync::atomic::{AtomicUsize, Ordering};
use core::{mem, usize};
use spin::RwLock;

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

/// UserScheme has to be wrapped
pub struct UserScheme {
    next_id: AtomicUsize,
    todo: RwLock<VecDeque<Packet>>,
    done: RwLock<BTreeMap<usize, usize>>
}

impl UserScheme {
    fn call(&self, a: Call, b: usize, c: usize, d: usize) -> Result<usize> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);

        self.todo.write().push_back(Packet {
            id: id,
            a: a as usize,
            b: b,
            c: c,
            d: d
        });

        loop {
            if let Some(a) = self.done.write().remove(&id) {
                return convert_to_result(a);
            }

            unsafe { context::switch(); }
        }
    }
}

impl Scheme for UserScheme {
    fn open(&self, path: &[u8], flags: usize) -> Result<usize> {
        self.call(Call::Open, path.as_ptr() as usize, path.len(), flags)
    }

    fn dup(&self, file: usize) -> Result<usize> {
        if file == usize::MAX {
            Ok(file)
        } else {
            self.call(Call::Dup, file, 0, 0)
        }
    }

    fn read(&self, file: usize, buf: &mut [u8]) -> Result<usize> {
        if file == usize::MAX {
            let packet_size = mem::size_of::<Packet>();
            let len = buf.len()/packet_size;
            if len > 0 {
                loop {
                    let mut i = 0;
                    {
                        let mut todo = self.todo.write();
                        while ! todo.is_empty() && i < len {
                            unsafe { *(buf.as_mut_ptr() as *mut Packet).offset(i as isize) = todo.pop_front().unwrap(); }
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
        } else {
            self.call(Call::Read, file, buf.as_mut_ptr() as usize, buf.len())
        }
    }

    fn write(&self, file: usize, buf: &[u8]) -> Result<usize> {
        if file == usize::MAX {
            let packet_size = mem::size_of::<Packet>();
            let len = buf.len()/packet_size;
            let mut i = 0;
            while i < len {
                let packet = unsafe { *(buf.as_ptr() as *const Packet).offset(i as isize) };
                self.done.write().insert(packet.id, packet.a);

                i += 1;
            }

            Ok(i * packet_size)
        } else {
            self.call(Call::Write, file, buf.as_ptr() as usize, buf.len())
        }
    }

    fn fsync(&self, file: usize) -> Result<()> {
        if file == usize::MAX {
            Ok(())
        } else {
            self.call(Call::FSync, file, 0, 0).and(Ok(()))
        }
    }

    fn close(&self, file: usize) -> Result<()> {
        if file == usize::MAX {
            println!("Close user scheme");
            Ok(())
        } else {
            self.call(Call::Close, file, 0, 0).and(Ok(()))
        }
    }
}
