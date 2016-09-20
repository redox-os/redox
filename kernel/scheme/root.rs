use alloc::arc::Arc;
use alloc::boxed::Box;
use collections::BTreeMap;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::str;
use spin::RwLock;

use syscall::{Error, Result};
use scheme::{self, Scheme};
use scheme::user::{UserInner, UserScheme};

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
    fn open(&self, path: &[u8], flags: usize) -> Result<usize> {
        let inner = {
            let mut schemes = scheme::schemes_mut();
            if schemes.get_name(path).is_some() {
                return Err(Error::FileExists);
            }
            let inner = Arc::new(UserInner::new());
            schemes.insert(path.to_vec().into_boxed_slice(), Arc::new(Box::new(UserScheme::new(Arc::downgrade(&inner))))).expect("failed to insert user scheme");
            inner
        };

        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        self.handles.write().insert(id, inner);

        Ok(id)
    }

    fn dup(&self, file: usize) -> Result<usize> {
        let inner = {
            let handles = self.handles.read();
            let inner = handles.get(&file).ok_or(Error::BadFile)?;
            inner.clone()
        };

        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        self.handles.write().insert(id, inner);

        Ok(id)
    }

    fn read(&self, file: usize, buf: &mut [u8]) -> Result<usize> {
        let inner = {
            let handles = self.handles.read();
            let inner = handles.get(&file).ok_or(Error::BadFile)?;
            inner.clone()
        };

        inner.read(buf)
    }

    fn write(&self, file: usize, buf: &[u8]) -> Result<usize> {
        let inner = {
            let handles = self.handles.read();
            let inner = handles.get(&file).ok_or(Error::BadFile)?;
            inner.clone()
        };

        inner.write(buf)
    }

    fn fsync(&self, _file: usize) -> Result<()> {
        Ok(())
    }

    fn close(&self, file: usize) -> Result<()> {
        self.handles.write().remove(&file).ok_or(Error::BadFile).and(Ok(()))
    }
}
