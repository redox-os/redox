//! Context management

use alloc::arc::Arc;
use collections::{BTreeMap, Vec};
use spin::{Once, RwLock, RwLockReadGuard, RwLockWriteGuard};

/// File operations
pub mod file;

/// Maximum context files
pub const CONTEXT_MAX_FILES: usize = 65536;

/// Context ID
pub type ContextId = u16;

/// Context list type
pub type ContextList = BTreeMap<ContextId, Arc<RwLock<Context>>>;

/// Contexts list
static CONTEXTS: Once<RwLock<ContextList>> = Once::new();

/// Initialize contexts, called if needed
fn init_contexts() -> RwLock<ContextList> {
    let mut map: ContextList = BTreeMap::new();
    map.insert(0, Arc::new(RwLock::new(Context::new())));
    RwLock::new(map)
}

/// Get the global schemes list, const
pub fn contexts() -> RwLockReadGuard<'static, ContextList> {
    CONTEXTS.call_once(init_contexts).read()
}

/// Get the global schemes list, mutable
pub fn contexts_mut() -> RwLockWriteGuard<'static, ContextList> {
    CONTEXTS.call_once(init_contexts).write()
}

/// A context, which identifies either a process or a thread
#[derive(Clone, Debug)]
pub struct Context {
    /// The open files in the scheme
    pub files: Vec<Option<file::File>>
}

impl Context {
    pub fn new() -> Context {
        Context {
            files: Vec::new()
        }
    }

    /// Add a file to the lowest available slot.
    /// Return the file descriptor number or None if no slot was found
    pub fn add_file(&mut self, file: file::File) -> Option<usize> {
        for (i, mut file_option) in self.files.iter_mut().enumerate() {
            if file_option.is_none() {
                *file_option = Some(file);
                return Some(i);
            }
        }
        let len = self.files.len();
        if len < CONTEXT_MAX_FILES {
            self.files.push(Some(file));
            Some(len)
        } else {
            None
        }
    }
}
