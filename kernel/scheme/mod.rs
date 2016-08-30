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

use syscall::Result;

use self::debug::DebugScheme;

pub use self::fd::Fd;

/// Debug scheme
pub mod debug;
mod fd;

/// Scheme list type
pub type SchemeList = BTreeMap<Box<[u8]>, Arc<Mutex<Box<Scheme + Send>>>>;

/// Schemes list
static SCHEMES: Once<RwLock<SchemeList>> = Once::new();

/// Initialize schemes, called if needed
fn init_schemes() -> RwLock<SchemeList> {
    let mut map: SchemeList = BTreeMap::new();
    map.insert(Box::new(*b"debug"), Arc::new(Mutex::new(Box::new(DebugScheme))));
    RwLock::new(map)
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

    /// Read from some file descriptor into the `buffer`
    ///
    /// Returns the number of bytes read
    fn read(&mut self, fd: Fd, buffer: &mut [u8]) -> Result<usize>;

    /// Write the `buffer` to the file descriptor
    ///
    /// Returns the number of bytes written
    fn write(&mut self, fd: Fd, buffer: &[u8]) -> Result<usize>;

    /// Close the file descriptor
    fn close(&mut self, fd: Fd) -> Result<()>;
}
