use alloc::arc::Arc;
use alloc::boxed::Box;
use collections::BTreeMap;
use core::sync::atomic::{AtomicUsize, Ordering};
use spin::RwLock;

use context;
use syscall::error::*;
use syscall::scheme::Scheme;
use scheme::{self, SchemeNamespace, SchemeId};
use scheme::user::{UserInner, UserScheme};

pub struct RootScheme {
    scheme_ns: SchemeNamespace,
    scheme_id: SchemeId,
    next_id: AtomicUsize,
    handles: RwLock<BTreeMap<usize, Arc<UserInner>>>
}

impl RootScheme {
    pub fn new(scheme_ns: SchemeNamespace, scheme_id: SchemeId) -> RootScheme {
        RootScheme {
            scheme_ns: scheme_ns,
            scheme_id: scheme_id,
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
                let inner = Arc::new(UserInner::new(self.scheme_id, id, flags, context));
                schemes.insert(self.scheme_ns, path.to_vec().into_boxed_slice(), |scheme_id| {
                    inner.scheme_id.store(scheme_id, Ordering::SeqCst);
                    Arc::new(Box::new(UserScheme::new(Arc::downgrade(&inner))))
                })?;
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
