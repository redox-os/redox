//! Context management

use collections::{BTreeMap, Vec};
use core::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT, Ordering};
use spin::{Once, RwLock, RwLockReadGuard, RwLockWriteGuard};

use syscall::{Error, Result};

/// File operations
pub mod file;

/// Limit on number of contexts
pub const CONTEXT_MAX_CONTEXTS: usize = 65536;

/// Maximum context files
pub const CONTEXT_MAX_FILES: usize = 65536;

/// Context list type
pub struct ContextList {
    map: BTreeMap<usize, RwLock<Context>>,
    next_id: usize
}

impl ContextList {
    pub fn new() -> Self {
        ContextList {
            map: BTreeMap::new(),
            next_id: 1
        }
    }

    pub fn get(&self, id: usize) -> Option<&RwLock<Context>> {
        self.map.get(&id)
    }

    pub fn current(&self) -> Option<&RwLock<Context>> {
        self.map.get(&CONTEXT_ID.load(Ordering::SeqCst))
    }

    pub fn new_context(&mut self) -> Result<&RwLock<Context>> {
        if self.next_id >= CONTEXT_MAX_CONTEXTS {
            self.next_id = 1;
        }

        while self.map.contains_key(&self.next_id) {
            self.next_id += 1;
        }

        if self.next_id >= CONTEXT_MAX_CONTEXTS {
            return Err(Error::TryAgain);
        }

        let id = self.next_id;
        self.next_id += 1;
        assert!(self.map.insert(id, RwLock::new(Context::new(id))).is_none());
        Ok(self.map.get(&id).expect("failed to insert new context"))
    }
}

/// Contexts list
static CONTEXTS: Once<RwLock<ContextList>> = Once::new();

#[thread_local]
static CONTEXT_ID: AtomicUsize = ATOMIC_USIZE_INIT;

pub fn init() {
    let mut contexts = contexts_mut();
    let context_lock = contexts.new_context().expect("could not initialize first context");
    let context = context_lock.read();
    CONTEXT_ID.store(context.id, Ordering::SeqCst);
}

/// Initialize contexts, called if needed
fn init_contexts() -> RwLock<ContextList> {
    RwLock::new(ContextList::new())
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
#[derive(Debug)]
pub struct Context {
    /// The ID of this context
    pub id: usize,
    /// The open files in the scheme
    pub files: Vec<Option<file::File>>
}

impl Context {
    pub fn new(id: usize) -> Context {
        Context {
            id: id,
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
