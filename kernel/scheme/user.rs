use alloc::arc::{Arc, Weak};
use collections::BTreeMap;
use core::sync::atomic::{AtomicU64, Ordering};
use core::{mem, slice, usize};
use spin::{Mutex, RwLock};

use arch;
use arch::paging::{InactivePageTable, Page, VirtualAddress, entry};
use arch::paging::temporary_page::TemporaryPage;
use context::{self, Context};
use context::memory::Grant;
use scheme::{AtomicSchemeId, ATOMIC_SCHEMEID_INIT, SchemeId};
use sync::{WaitQueue, WaitMap};
use syscall::data::{Packet, Stat};
use syscall::error::*;
use syscall::flag::{EVENT_READ, O_NONBLOCK};
use syscall::number::*;
use syscall::scheme::Scheme;

pub struct UserInner {
    root_id: SchemeId,
    handle_id: usize,
    flags: usize,
    pub scheme_id: AtomicSchemeId,
    next_id: AtomicU64,
    context: Weak<RwLock<Context>>,
    todo: WaitQueue<Packet>,
    fmap: Mutex<BTreeMap<u64, (Weak<RwLock<Context>>, usize)>>,
    done: WaitMap<u64, usize>
}

impl UserInner {
    pub fn new(root_id: SchemeId, handle_id: usize, flags: usize, context: Weak<RwLock<Context>>) -> UserInner {
        UserInner {
            root_id: root_id,
            handle_id: handle_id,
            flags: flags,
            scheme_id: ATOMIC_SCHEMEID_INIT,
            next_id: AtomicU64::new(1),
            context: context,
            todo: WaitQueue::new(),
            fmap: Mutex::new(BTreeMap::new()),
            done: WaitMap::new()
        }
    }

    pub fn call(&self, a: usize, b: usize, c: usize, d: usize) -> Result<usize> {
        let (pid, uid, gid) = {
            let contexts = context::contexts();
            let context_lock = contexts.current().ok_or(Error::new(ESRCH))?;
            let context = context_lock.read();
            (context.id, context.euid, context.egid)
        };

        self.call_inner(Packet {
            id: self.next_id.fetch_add(1, Ordering::SeqCst),
            pid: pid.into(),
            uid: uid,
            gid: gid,
            a: a,
            b: b,
            c: c,
            d: d
        })
    }

    fn call_inner(&self, packet: Packet) -> Result<usize> {
        let id = packet.id;

        let len = self.todo.send(packet);
        context::event::trigger(self.root_id, self.handle_id, EVENT_READ, mem::size_of::<Packet>() * len);

        Error::demux(self.done.receive(&id))
    }

    pub fn capture(&self, buf: &[u8]) -> Result<usize> {
        UserInner::capture_inner(&self.context, buf.as_ptr() as usize, buf.len(), false)
    }

    pub fn capture_mut(&self, buf: &mut [u8]) -> Result<usize> {
        UserInner::capture_inner(&self.context, buf.as_mut_ptr() as usize, buf.len(), true)
    }

    fn capture_inner(context_weak: &Weak<RwLock<Context>>, address: usize, size: usize, writable: bool) -> Result<usize> {
        if size == 0 {
            Ok(0)
        } else {
            let context_lock = context_weak.upgrade().ok_or(Error::new(ESRCH))?;
            let context = context_lock.read();

            let mut grants = context.grants.lock();

            let mut new_table = unsafe { InactivePageTable::from_address(context.arch.get_page_table()) };
            let mut temporary_page = TemporaryPage::new(Page::containing_address(VirtualAddress::new(arch::USER_TMP_GRANT_OFFSET)));

            let from_address = (address/4096) * 4096;
            let offset = address - from_address;
            let full_size = ((offset + size + 4095)/4096) * 4096;
            let mut to_address = arch::USER_GRANT_OFFSET;

            let mut flags = entry::PRESENT | entry::NO_EXECUTE | entry::USER_ACCESSIBLE;
            if writable {
                flags |= entry::WRITABLE;
            }

            for i in 0 .. grants.len() {
                let start = grants[i].start_address().get();
                if to_address + full_size < start {
                    grants.insert(i, Grant::map_inactive(
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

            grants.push(Grant::map_inactive(
                VirtualAddress::new(from_address),
                VirtualAddress::new(to_address),
                full_size,
                flags,
                &mut new_table,
                &mut temporary_page
            ));

            Ok(to_address + offset)
        }
    }

    pub fn release(&self, address: usize) -> Result<()> {
        if address == 0 {
            Ok(())
        } else {
            let context_lock = self.context.upgrade().ok_or(Error::new(ESRCH))?;
            let context = context_lock.read();

            let mut grants = context.grants.lock();

            let mut new_table = unsafe { InactivePageTable::from_address(context.arch.get_page_table()) };
            let mut temporary_page = TemporaryPage::new(Page::containing_address(VirtualAddress::new(arch::USER_TMP_GRANT_OFFSET)));

            for i in 0 .. grants.len() {
                let start = grants[i].start_address().get();
                let end = start + grants[i].size();
                if address >= start && address < end {
                    grants.remove(i).unmap_inactive(&mut new_table, &mut temporary_page);

                    return Ok(());
                }
            }

            Err(Error::new(EFAULT))
        }
    }

    pub fn read(&self, buf: &mut [u8]) -> Result<usize> {
        let packet_buf = unsafe { slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut Packet, buf.len()/mem::size_of::<Packet>()) };
        Ok(self.todo.receive_into(packet_buf, self.flags & O_NONBLOCK != O_NONBLOCK) * mem::size_of::<Packet>())
    }

    pub fn write(&self, buf: &[u8]) -> Result<usize> {
        let packet_size = mem::size_of::<Packet>();
        let len = buf.len()/packet_size;
        let mut i = 0;
        while i < len {
            let mut packet = unsafe { *(buf.as_ptr() as *const Packet).offset(i as isize) };
            if packet.id == 0 {
                match packet.a {
                    SYS_FEVENT => context::event::trigger(self.scheme_id.load(Ordering::SeqCst), packet.b, packet.c, packet.d),
                    _ => println!("Unknown scheme -> kernel message {}", packet.a)
                }
            } else {
                if let Some((context_weak, size)) = self.fmap.lock().remove(&packet.id) {
                    if let Ok(address) = Error::demux(packet.a) {
                        packet.a = Error::mux(UserInner::capture_inner(&context_weak, address, size, true));
                    }
                }

                self.done.send(packet.id, packet.a);
            }
            i += 1;
        }

        Ok(i * packet_size)
    }

    pub fn fevent(&self, _flags: usize) -> Result<usize> {
        Ok(self.handle_id)
    }

    pub fn fsync(&self) -> Result<usize> {
        Ok(0)
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
    fn open(&self, path: &[u8], flags: usize, _uid: u32, _gid: u32) -> Result<usize> {
        let inner = self.inner.upgrade().ok_or(Error::new(ENODEV))?;
        let address = inner.capture(path)?;
        let result = inner.call(SYS_OPEN, address, path.len(), flags);
        let _ = inner.release(address);
        result
    }

    fn mkdir(&self, path: &[u8], mode: u16, _uid: u32, _gid: u32) -> Result<usize> {
        let inner = self.inner.upgrade().ok_or(Error::new(ENODEV))?;
        let address = inner.capture(path)?;
        let result = inner.call(SYS_MKDIR, address, path.len(), mode as usize);
        let _ = inner.release(address);
        result
    }

    fn chmod(&self, path: &[u8], mode: u16, _uid: u32, _gid: u32) -> Result<usize> {
        let inner = self.inner.upgrade().ok_or(Error::new(ENODEV))?;
        let address = inner.capture(path)?;
        let result = inner.call(SYS_CHMOD, address, path.len(), mode as usize);
        let _ = inner.release(address);
        result
    }

    fn rmdir(&self, path: &[u8], _uid: u32, _gid: u32) -> Result<usize> {
        let inner = self.inner.upgrade().ok_or(Error::new(ENODEV))?;
        let address = inner.capture(path)?;
        let result = inner.call(SYS_RMDIR, address, path.len(), 0);
        let _ = inner.release(address);
        result
    }

    fn unlink(&self, path: &[u8], _uid: u32, _gid: u32) -> Result<usize> {
        let inner = self.inner.upgrade().ok_or(Error::new(ENODEV))?;
        let address = inner.capture(path)?;
        let result = inner.call(SYS_UNLINK, address, path.len(), 0);
        let _ = inner.release(address);
        result
    }

    fn dup(&self, file: usize, buf: &[u8]) -> Result<usize> {
        let inner = self.inner.upgrade().ok_or(Error::new(ENODEV))?;
        let address = inner.capture(buf)?;
        let result = inner.call(SYS_DUP, file, address, buf.len());
        let _ = inner.release(address);
        result
    }

    fn read(&self, file: usize, buf: &mut [u8]) -> Result<usize> {
        let inner = self.inner.upgrade().ok_or(Error::new(ENODEV))?;
        let address = inner.capture_mut(buf)?;
        let result = inner.call(SYS_READ, file, address, buf.len());
        let _ = inner.release(address);
        result
    }

    fn write(&self, file: usize, buf: &[u8]) -> Result<usize> {
        let inner = self.inner.upgrade().ok_or(Error::new(ENODEV))?;
        let address = inner.capture(buf)?;
        let result = inner.call(SYS_WRITE, file, address, buf.len());
        let _ = inner.release(address);
        result
    }

    fn seek(&self, file: usize, position: usize, whence: usize) -> Result<usize> {
        let inner = self.inner.upgrade().ok_or(Error::new(ENODEV))?;
        inner.call(SYS_LSEEK, file, position, whence)
    }

    fn fcntl(&self, file: usize, cmd: usize, arg: usize) -> Result<usize> {
        let inner = self.inner.upgrade().ok_or(Error::new(ENODEV))?;
        inner.call(SYS_FCNTL, file, cmd, arg)
    }

    fn fevent(&self, file: usize, flags: usize) -> Result<usize> {
        let inner = self.inner.upgrade().ok_or(Error::new(ENODEV))?;
        inner.call(SYS_FEVENT, file, flags, 0)
    }

    fn fmap(&self, file: usize, offset: usize, size: usize) -> Result<usize> {
        let inner = self.inner.upgrade().ok_or(Error::new(ENODEV))?;

        let (pid, uid, gid, context_lock) = {
            let contexts = context::contexts();
            let context_lock = contexts.current().ok_or(Error::new(ESRCH))?;
            let context = context_lock.read();
            (context.id, context.euid, context.egid, Arc::downgrade(&context_lock))
        };

        let id = inner.next_id.fetch_add(1, Ordering::SeqCst);

        inner.fmap.lock().insert(id, (context_lock, size));

        inner.call_inner(Packet {
            id: id,
            pid: pid.into(),
            uid: uid,
            gid: gid,
            a: SYS_FMAP,
            b: file,
            c: offset,
            d: size
        })
    }

    fn fpath(&self, file: usize, buf: &mut [u8]) -> Result<usize> {
        let inner = self.inner.upgrade().ok_or(Error::new(ENODEV))?;
        let address = inner.capture_mut(buf)?;
        let result = inner.call(SYS_FPATH, file, address, buf.len());
        let _ = inner.release(address);
        result
    }

    fn fstat(&self, file: usize, stat: &mut Stat) -> Result<usize> {
        let inner = self.inner.upgrade().ok_or(Error::new(ENODEV))?;
        let address = inner.capture_mut(stat)?;
        let result = inner.call(SYS_FSTAT, file, address, 0);
        let _ = inner.release(address);
        result
    }

    fn fsync(&self, file: usize) -> Result<usize> {
        let inner = self.inner.upgrade().ok_or(Error::new(ENODEV))?;
        inner.call(SYS_FSYNC, file, 0, 0)
    }

    fn ftruncate(&self, file: usize, len: usize) -> Result<usize> {
        let inner = self.inner.upgrade().ok_or(Error::new(ENODEV))?;
        inner.call(SYS_FTRUNCATE, file, len, 0)
    }

    fn close(&self, file: usize) -> Result<usize> {
        let inner = self.inner.upgrade().ok_or(Error::new(ENODEV))?;
        inner.call(SYS_CLOSE, file, 0, 0)
    }
}
