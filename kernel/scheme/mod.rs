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

use spin::{Once, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};

use syscall::{Error, Result};

use self::debug::DebugScheme;
use self::initfs::InitFsScheme;

/// Debug scheme
pub mod debug;

/// InitFS scheme
pub mod initfs;

/// Limit on number of schemes
pub const SCHEME_MAX_SCHEMES: usize = 65536;

/// Scheme list type
pub struct SchemeList {
    map: BTreeMap<usize, Arc<Mutex<Box<Scheme + Send>>>>,
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
    pub fn get(&self, id: usize) -> Option<&Arc<Mutex<Box<Scheme + Send>>>> {
        self.map.get(&id)
    }

    pub fn get_name(&self, name: &[u8]) -> Option<(usize, &Arc<Mutex<Box<Scheme + Send>>>)> {
        if let Some(&id) = self.names.get(name) {
            self.get(id).map(|scheme| (id, scheme))
        } else {
            None
        }
    }

    /// Create a new context.
    pub fn insert(&mut self, name: Box<[u8]>, scheme: Arc<Mutex<Box<Scheme + Send>>>) -> Result<&Arc<Mutex<Box<Scheme + Send>>>> {
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
    list.insert(Box::new(*b"debug"), Arc::new(Mutex::new(Box::new(DebugScheme)))).expect("failed to insert debug: scheme");
    list.insert(Box::new(*b"initfs"), Arc::new(Mutex::new(Box::new(InitFsScheme::new())))).expect("failed to insert initfs: scheme");
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
    fn open(&mut self, path: &[u8], flags: usize) -> Result<usize>;

    /// Duplicate an open file descriptor
    ///
    /// Returns a file descriptor or an error
    fn dup(&mut self, file: usize) -> Result<usize>;

    /// Read from some file descriptor into the `buffer`
    ///
    /// Returns the number of bytes read
    fn read(&mut self, file: usize, buffer: &mut [u8]) -> Result<usize>;

    /// Write the `buffer` to the file descriptor
    ///
    /// Returns the number of bytes written
    fn write(&mut self, file: usize, buffer: &[u8]) -> Result<usize>;

    /// Close the file descriptor
    fn close(&mut self, file: usize) -> Result<()>;
}
