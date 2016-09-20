use alloc::arc::Weak;
use collections::{BTreeMap, VecDeque};
use core::sync::atomic::{AtomicUsize, Ordering};
use core::{mem, usize};
use spin::{Mutex, RwLock};

use arch;
use arch::paging::{InactivePageTable, Page, VirtualAddress, entry};
use arch::paging::temporary_page::TemporaryPage;
use context::{self, Context};
use context::memory::Grant;
use syscall::{convert_to_result, Call, Error, Result};

use super::Scheme;

#[derive(Copy, Clone, Debug, Default)]
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
    context: Weak<RwLock<Context>>,
    todo: Mutex<VecDeque<Packet>>,
    done: Mutex<BTreeMap<usize, usize>>
}

impl UserInner {
    pub fn new(context: Weak<RwLock<Context>>) -> UserInner {
        UserInner {
            next_id: AtomicUsize::new(0),
            context: context,
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
            {
                let mut done = self.done.lock();
                if let Some(a) = done.remove(&id) {
                    return convert_to_result(a);
                }
            }

            unsafe { context::switch(); }
        }
    }

    pub fn capture(&self, buf: &[u8]) -> Result<usize> {
        self.capture_inner(buf.as_ptr() as usize, buf.len(), false)
    }

    pub fn capture_mut(&self, buf: &mut [u8]) -> Result<usize> {
        self.capture_inner(buf.as_mut_ptr() as usize, buf.len(), true)
    }

    fn capture_inner(&self, address: usize, size: usize, writable: bool) -> Result<usize> {
        let context_lock = self.context.upgrade().ok_or(Error::NoProcess)?;
        let context = context_lock.read();

        let mut grants = context.grants.lock();

        let mut new_table = unsafe { InactivePageTable::from_address(context.arch.get_page_table()) };
        let mut temporary_page = TemporaryPage::new(Page::containing_address(VirtualAddress::new(arch::USER_TMP_GRANT_OFFSET)));

        let from_address = (address/4096) * 4096;
        let offset = address - from_address;
        let full_size = ((offset + size + 4095)/4096) * 4096;
        let mut to_address = arch::USER_GRANT_OFFSET;

        let mut flags = entry::PRESENT | entry::NO_EXECUTE;
        if writable {
            flags |= entry::WRITABLE;
        }

        for i in 0 .. grants.len() {
            let start = grants[i].start_address().get();
            if to_address + full_size < start {
                grants.insert(i, Grant::new(
                    VirtualAddress::new(from_address),
                    VirtualAddress::new(to_address),
                    full_size,
                    flags,
                    &mut new_table,
                    &mut temporary_page
                ));

                return Ok(to_address + offset);
            } else {
                let pages = (grants[i].size() + 4095) / 4096;
                let end = start + pages * 4096;
                to_address = end;
            }
        }

        grants.push(Grant::new(
            VirtualAddress::new(from_address),
            VirtualAddress::new(to_address),
            full_size,
            flags,
            &mut new_table,
            &mut temporary_page
        ));

        return Ok(to_address + offset);
    }

    pub fn release(&self, address: usize) -> Result<()> {
        let context_lock = self.context.upgrade().ok_or(Error::NoProcess)?;
        let context = context_lock.read();

        let mut grants = context.grants.lock();

        let mut new_table = unsafe { InactivePageTable::from_address(context.arch.get_page_table()) };
        let mut temporary_page = TemporaryPage::new(Page::containing_address(VirtualAddress::new(arch::USER_TMP_GRANT_OFFSET)));

        for i in 0 .. grants.len() {
            let start = grants[i].start_address().get();
            let end = start + grants[i].size();
            if address >= start && address < end {
                grants.remove(i).destroy(&mut new_table, &mut temporary_page);

                return Ok(());
            }
        }

        Err(Error::Fault)
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
        let address = inner.capture(path)?;
        let result = inner.call(Call::Open, address, path.len(), flags);
        let _ = inner.release(address);
        result
    }

    fn dup(&self, file: usize) -> Result<usize> {
        let inner = self.inner.upgrade().ok_or(Error::NoDevice)?;
        inner.call(Call::Dup, file, 0, 0)
    }

    fn read(&self, file: usize, buf: &mut [u8]) -> Result<usize> {
        let inner = self.inner.upgrade().ok_or(Error::NoDevice)?;
        let address = inner.capture_mut(buf)?;
        let result = inner.call(Call::Read, file, address, buf.len());
        let _ = inner.release(address);
        result
    }

    fn write(&self, file: usize, buf: &[u8]) -> Result<usize> {
        let inner = self.inner.upgrade().ok_or(Error::NoDevice)?;
        let address = inner.capture(buf)?;
        let result = inner.call(Call::Write, file, buf.as_ptr() as usize, buf.len());
        let _ = inner.release(address);
        result
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
