use alloc::arc::Arc;
use alloc::boxed::Box;
use collections::BTreeMap;
use core::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT, Ordering};
use spin::RwLock;

use context;
use syscall::error::*;
use syscall::scheme::Scheme;
use scheme;
use scheme::user::{UserInner, UserScheme};

pub static ROOT_SCHEME_ID: AtomicUsize = ATOMIC_USIZE_INIT;

pub struct RootScheme {
    next_id: AtomicUsize,
    handles: RwLock<BTreeMap<usize, Arc<UserInner>>>
}

impl RootScheme {
    pub fn new() -> RootScheme {
        RootScheme {
            next_id: AtomicUsize::new(0),
            handles: RwLock::new(BTreeMap::new())
        }
    }
}

impl Scheme for RootScheme {
    fn open(&self, path: &[u8], flags: usize, uid: u32, _gid: u32) -> Result<usize> {
        if uid == 0 {
            let context = {
                let contexts = context::contexts();
                let context = contexts.current().ok_or(Error::new(ESRCH))?;
                Arc::downgrade(&context)
            };

            let id = self.next_id.fetch_add(1, Ordering::SeqCst);

            let inner = {
                let mut schemes = scheme::schemes_mut();
                if schemes.get_name(path).is_some() {
                    return Err(Error::new(EEXIST));
                }
                let inner = Arc::new(UserInner::new(id, flags, context));
                let scheme_id = schemes.insert(path.to_vec().into_boxed_slice(), Arc::new(Box::new(UserScheme::new(Arc::downgrade(&inner))))).expect("failed to insert user scheme");
                inner.scheme_id.store(scheme_id, Ordering::SeqCst);
                inner
            };

            self.handles.write().insert(id, inner);

            Ok(id)
        } else {
            Err(Error::new(EACCES))
        }
    }

    fn dup(&self, file: usize, _buf: &[u8]) -> Result<usize> {
        let mut handles = self.handles.write();
        let inner = {
            let inner = handles.get(&file).ok_or(Error::new(EBADF))?;
            inner.clone()
        };

        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        handles.insert(id, inner);

        Ok(id)
    }

    fn read(&self, file: usize, buf: &mut [u8]) -> Result<usize> {
        let inner = {
            let handles = self.handles.read();
            let inner = handles.get(&file).ok_or(Error::new(EBADF))?;
            inner.clone()
        };

        inner.read(buf)
    }

    fn write(&self, file: usize, buf: &[u8]) -> Result<usize> {
        let inner = {
            let handles = self.handles.read();
            let inner = handles.get(&file).ok_or(Error::new(EBADF))?;
            inner.clone()
        };

        inner.write(buf)
    }

    fn fevent(&self, file: usize, flags: usize) -> Result<usize> {
        let inner = {
            let handles = self.handles.read();
            let inner = handles.get(&file).ok_or(Error::new(EBADF))?;
            inner.clone()
        };

        inner.fevent(flags)
    }

    fn fsync(&self, file: usize) -> Result<usize> {
        let inner = {
            let handles = self.handles.read();
            let inner = handles.get(&file).ok_or(Error::new(EBADF))?;
            inner.clone()
        };

        inner.fsync()
    }

    fn close(&self, file: usize) -> Result<usize> {
        self.handles.write().remove(&file).ok_or(Error::new(EBADF)).and(Ok(0))
    }
}
