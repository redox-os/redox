//! Context management

use alloc::boxed::Box;
use collections::{BTreeMap, Vec};
use core::mem;
use core::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT, Ordering};
use spin::{Once, RwLock, RwLockReadGuard, RwLockWriteGuard};

use arch;
use arch::context::Context as ArchContext;
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
    /// Create a new context list.
    pub fn new() -> Self {
        ContextList {
            map: BTreeMap::new(),
            next_id: 1
        }
    }

    /// Get the nth context.
    pub fn get(&self, id: usize) -> Option<&RwLock<Context>> {
        self.map.get(&id)
    }

    /// Get the current context.
    pub fn current(&self) -> Option<&RwLock<Context>> {
        self.map.get(&CONTEXT_ID.load(Ordering::SeqCst))
    }

    /// Create a new context.
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

        Ok(self.map.get(&id).expect("Failed to insert new context. ID is out of bounds."))
    }

    /// Spawn a context from a function.
    pub fn spawn(&mut self, func: extern fn()) -> Result<&RwLock<Context>> {
        let context_lock = self.new_context()?;
        {
            let mut context = context_lock.write();
            let mut stack = Box::new([0; 65536]);
            let offset = stack.len() - mem::size_of::<usize>();
            unsafe {
                let offset = stack.len() - mem::size_of::<usize>();
                let func_ptr = stack.as_mut_ptr().offset(offset as isize);
                *(func_ptr as *mut usize) = func as usize;
            }
            context.arch.set_stack(stack.as_ptr() as usize + offset);
            context.kstack = Some(stack);
        }
        Ok(context_lock)
    }
}

/// Contexts list
static CONTEXTS: Once<RwLock<ContextList>> = Once::new();

#[thread_local]
static CONTEXT_ID: AtomicUsize = ATOMIC_USIZE_INIT;

pub fn init() {
    let mut contexts = contexts_mut();
    let context_lock = contexts.new_context().expect("could not initialize first context");
    let mut context = context_lock.write();
    context.running = true;
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

/// Switch to the next context
///
/// # Safety
///
/// Do not call this while holding locks!
pub unsafe fn switch() {
    use core::ops::DerefMut;

    // Set the global lock to avoid the unsafe operations below from causing issues
    while arch::context::CONTEXT_SWITCH_LOCK.compare_and_swap(false, true, Ordering::SeqCst) {
        arch::interrupt::pause();
    }

    let from_ptr = {
        let contexts = contexts();
        let context_lock = contexts.current().expect("context::switch: Not inside of context");
        let mut context = context_lock.write();
        context.deref_mut() as *mut Context
    };

    let mut to_ptr = 0 as *mut Context;

    for (_pid, context_lock) in contexts().map.iter() {
        let mut context = context_lock.write();
        if ! context.running && ! context.blocked {
            to_ptr = context.deref_mut() as *mut Context;
            break;
        }
    }

    if to_ptr as usize == 0 {
        // TODO: Sleep, wait for interrupt
        // Unset global lock if no context found
        arch::context::CONTEXT_SWITCH_LOCK.store(false, Ordering::SeqCst);
        return;
    }

    (&mut *from_ptr).running = false;
    (&mut *to_ptr).running = true;
    CONTEXT_ID.store((&mut *to_ptr).id, Ordering::SeqCst);

    (&mut *from_ptr).arch.switch_to(&mut (&mut *to_ptr).arch);
}

/// A context, which identifies either a process or a thread
#[derive(Debug)]
pub struct Context {
    /// The ID of this context
    pub id: usize,
    /// Running or not
    pub running: bool,
    /// Blocked or not
    pub blocked: bool,
    /// The architecture specific context
    pub arch: ArchContext,
    /// Kernel stack
    pub kstack: Option<Box<[u8]>>,
    /// The open files in the scheme
    pub files: Vec<Option<file::File>>
}

impl Context {
    /// Create a new context
    pub fn new(id: usize) -> Context {
        Context {
            id: id,
            running: false,
            blocked: true,
            arch: ArchContext::new(),
            kstack: None,
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

    /// Get a file
    pub fn get_file(&self, i: usize) -> Option<file::File> {
        if i < self.files.len() {
            self.files[i]
        } else {
            None
        }
    }

    /// Remove a file
    // TODO: adjust files vector to smaller size if possible
    pub fn remove_file(&mut self, i: usize) -> Option<file::File> {
        if i < self.files.len() {
            self.files[i].take()
        } else {
            None
        }
    }
}
