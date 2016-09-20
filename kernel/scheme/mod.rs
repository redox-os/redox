//! # Schemes
//! A scheme is a primitive for handling filesystem syscalls in Redox.
//! Schemes accept paths from the kernel for `open`, and file descriptors that they generate
//! are then passed for operations like `close`, `read`, `write`, etc.
//!
//! The kernel validates paths and file descriptors before they are passed to schemes,
//! also stripping the scheme identifier of paths if necessary.

use alloc::arc::Arc;
use alloc::boxed::Box;

use collections::BTreeMap;

use spin::{Once, RwLock, RwLockReadGuard, RwLockWriteGuard};

use syscall::{Error, Result};

use self::debug::DebugScheme;
use self::env::EnvScheme;
use self::initfs::InitFsScheme;
use self::irq::IrqScheme;

/// Debug scheme
pub mod debug;

/// Environmental variables
pub mod env;

/// InitFS scheme
pub mod initfs;

/// IRQ handling
pub mod irq;

/// Userspace schemes
pub mod user;

/// Limit on number of schemes
pub const SCHEME_MAX_SCHEMES: usize = 65536;

/// Scheme list type
pub struct SchemeList {
    map: BTreeMap<usize, Arc<Box<Scheme + Send + Sync>>>,
    names: BTreeMap<Box<[u8]>, usize>,
    next_id: usize
}

impl SchemeList {
    /// Create a new scheme list.
    pub fn new() -> Self {
        SchemeList {
            map: BTreeMap::new(),
            names: BTreeMap::new(),
            next_id: 1
        }
    }

    /// Get the nth scheme.
    pub fn get(&self, id: usize) -> Option<&Arc<Box<Scheme + Send + Sync>>> {
        self.map.get(&id)
    }

    pub fn get_name(&self, name: &[u8]) -> Option<(usize, &Arc<Box<Scheme + Send + Sync>>)> {
        if let Some(&id) = self.names.get(name) {
            self.get(id).map(|scheme| (id, scheme))
        } else {
            None
        }
    }

    /// Create a new scheme.
    pub fn insert(&mut self, name: Box<[u8]>, scheme: Arc<Box<Scheme + Send + Sync>>) -> Result<&Arc<Box<Scheme + Send + Sync>>> {
        if self.names.contains_key(&name) {
            return Err(Error::FileExists);
        }

        if self.next_id >= SCHEME_MAX_SCHEMES {
            self.next_id = 1;
        }

        while self.map.contains_key(&self.next_id) {
            self.next_id += 1;
        }

        if self.next_id >= SCHEME_MAX_SCHEMES {
            return Err(Error::TryAgain);
        }

        let id = self.next_id;
        self.next_id += 1;

        assert!(self.map.insert(id, scheme).is_none());
        assert!(self.names.insert(name, id).is_none());

        Ok(self.map.get(&id).expect("Failed to insert new scheme. ID is out of bounds."))
    }
}

/// Schemes list
static SCHEMES: Once<RwLock<SchemeList>> = Once::new();

/// Initialize schemes, called if needed
fn init_schemes() -> RwLock<SchemeList> {
    let mut list: SchemeList = SchemeList::new();
    list.insert(Box::new(*b"debug"), Arc::new(Box::new(DebugScheme))).expect("failed to insert debug: scheme");
    list.insert(Box::new(*b"env"), Arc::new(Box::new(EnvScheme::new()))).expect("failed to insert env: scheme");
    list.insert(Box::new(*b"initfs"), Arc::new(Box::new(InitFsScheme::new()))).expect("failed to insert initfs: scheme");
    list.insert(Box::new(*b"irq"), Arc::new(Box::new(IrqScheme))).expect("failed to insert irq: scheme");
    RwLock::new(list)
}

/// Get the global schemes list, const
pub fn schemes() -> RwLockReadGuard<'static, SchemeList> {
    SCHEMES.call_once(init_schemes).read()
}

/// Get the global schemes list, mutable
pub fn schemes_mut() -> RwLockWriteGuard<'static, SchemeList> {
    SCHEMES.call_once(init_schemes).write()
}

/// A scheme trait, implemented by a scheme handler
pub trait Scheme {
    /// Open the file at `path` with `flags`.
    ///
    /// Returns a file descriptor or an error
    fn open(&self, path: &[u8], flags: usize) -> Result<usize>;

    /// Duplicate an open file descriptor
    ///
    /// Returns a file descriptor or an error
    fn dup(&self, file: usize) -> Result<usize>;

    /// Read from some file descriptor into the `buffer`
    ///
    /// Returns the number of bytes read
    fn read(&self, file: usize, buffer: &mut [u8]) -> Result<usize>;

    /// Write the `buffer` to the file descriptor
    ///
    /// Returns the number of bytes written
    fn write(&self, file: usize, buffer: &[u8]) -> Result<usize>;

    /// Sync the file descriptor
    fn fsync(&self, file: usize) -> Result<()>;

    /// Close the file descriptor
    fn close(&self, file: usize) -> Result<()>;
}
