use alloc::arc::Arc;
use alloc::boxed::Box;
use collections::Vec;
use spin::Mutex;

use arch;
use super::file::File;
use super::memory::{Memory, SharedMemory};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Status {
    Runnable,
    Blocked,
    Exited
}

/// A context, which identifies either a process or a thread
#[derive(Debug)]
pub struct Context {
    /// The ID of this context
    pub id: usize,
    /// Status of context
    pub status: Status,
    /// Context running or not
    pub running: bool,
    /// The architecture specific context
    pub arch: arch::context::Context,
    /// Kernel stack
    pub kstack: Option<Box<[u8]>>,
    /// Executable image
    pub image: Vec<SharedMemory>,
    /// User heap
    pub heap: Option<SharedMemory>,
    /// User stack
    pub stack: Option<Memory>,
    /// The open files in the scheme
    pub files: Arc<Mutex<Vec<Option<File>>>>
}

impl Context {
    /// Create a new context
    pub fn new(id: usize) -> Context {
        Context {
            id: id,
            status: Status::Blocked,
            running: false,
            arch: arch::context::Context::new(),
            kstack: None,
            image: Vec::new(),
            heap: None,
            stack: None,
            files: Arc::new(Mutex::new(Vec::new()))
        }
    }

    /// Add a file to the lowest available slot.
    /// Return the file descriptor number or None if no slot was found
    pub fn add_file(&mut self, file: File) -> Option<usize> {
        let mut files = self.files.lock();
        for (i, mut file_option) in files.iter_mut().enumerate() {
            if file_option.is_none() {
                *file_option = Some(file);
                return Some(i);
            }
        }
        let len = files.len();
        if len < super::CONTEXT_MAX_FILES {
            files.push(Some(file));
            Some(len)
        } else {
            None
        }
    }

    /// Get a file
    pub fn get_file(&self, i: usize) -> Option<File> {
        let files = self.files.lock();
        if i < files.len() {
            files[i]
        } else {
            None
        }
    }

    /// Remove a file
    // TODO: adjust files vector to smaller size if possible
    pub fn remove_file(&mut self, i: usize) -> Option<File> {
        let mut files = self.files.lock();
        if i < files.len() {
            files[i].take()
        } else {
            None
        }
    }
}
