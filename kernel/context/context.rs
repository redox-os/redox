use alloc::arc::Arc;
use alloc::boxed::Box;
use collections::{BTreeMap, Vec};
use spin::Mutex;

use arch;
use context::file::File;
use context::memory::{Grant, Memory, SharedMemory};
use syscall::data::Event;
use sync::{WaitCondition, WaitQueue};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Status {
    Runnable,
    Blocked,
    Exited(usize)
}

/// A context, which identifies either a process or a thread
#[derive(Debug)]
pub struct Context {
    /// The ID of this context
    pub id: usize,
    /// The ID of the parent context
    pub ppid: usize,
    /// The real user id
    pub ruid: u32,
    /// The real group id
    pub rgid: u32,
    /// The effective user id
    pub euid: u32,
    /// The effective group id
    pub egid: u32,
    /// Status of context
    pub status: Status,
    /// Context running or not
    pub running: bool,
    /// Context is halting parent
    pub vfork: bool,
    /// Context is being waited on
    pub waitpid: Arc<WaitCondition>,
    /// Context should wake up at specified time
    pub wake: Option<(u64, u64)>,
    /// The architecture specific context
    pub arch: arch::context::Context,
    /// Kernel FX
    pub kfx: Option<Box<[u8]>>,
    /// Kernel stack
    pub kstack: Option<Box<[u8]>>,
    /// Executable image
    pub image: Vec<SharedMemory>,
    /// User heap
    pub heap: Option<SharedMemory>,
    /// User stack
    pub stack: Option<Memory>,
    /// User grants
    pub grants: Arc<Mutex<Vec<Grant>>>,
    /// The name of the context
    pub name: Arc<Mutex<Vec<u8>>>,
    /// The current working directory
    pub cwd: Arc<Mutex<Vec<u8>>>,
    /// Kernel events
    pub events: Arc<WaitQueue<Event>>,
    /// The process environment
    pub env: Arc<Mutex<BTreeMap<Box<[u8]>, Arc<Mutex<Vec<u8>>>>>>,
    /// The open files in the scheme
    pub files: Arc<Mutex<Vec<Option<File>>>>
}

impl Context {
    /// Create a new context
    pub fn new(id: usize) -> Context {
        Context {
            id: id,
            ppid: 0,
            ruid: 0,
            rgid: 0,
            euid: 0,
            egid: 0,
            status: Status::Blocked,
            running: false,
            vfork: false,
            waitpid: Arc::new(WaitCondition::new()),
            wake: None,
            arch: arch::context::Context::new(),
            kfx: None,
            kstack: None,
            image: Vec::new(),
            heap: None,
            stack: None,
            grants: Arc::new(Mutex::new(Vec::new())),
            name: Arc::new(Mutex::new(Vec::new())),
            cwd: Arc::new(Mutex::new(Vec::new())),
            events: Arc::new(WaitQueue::new()),
            env: Arc::new(Mutex::new(BTreeMap::new())),
            files: Arc::new(Mutex::new(Vec::new()))
        }
    }

    pub fn canonicalize(&self, path: &[u8]) -> Vec<u8> {
        if path.iter().position(|&b| b == b':').is_none() {
            let cwd = self.cwd.lock();
            if path == b"." {
                cwd.clone()
            } else if path == b".." {
                cwd[..cwd[..cwd.len() - 1]
                                   .iter().rposition(|&b| b == b'/' || b == b':')
                                   .map_or(cwd.len(), |i| i + 1)]
                   .to_vec()
            } else if path.starts_with(b"./") {
                let mut canon = cwd.clone();
                if ! canon.ends_with(b"/") {
                    canon.push(b'/');
                }
                canon.extend_from_slice(&path[2..]);
                canon
            } else if path.starts_with(b"../") {
                let mut canon = cwd[..cwd[..cwd.len() - 1]
                                   .iter().rposition(|&b| b == b'/' || b == b':')
                                   .map_or(cwd.len(), |i| i + 1)]
                   .to_vec();
                canon.extend_from_slice(&path[3..]);
                canon
            } else if path.starts_with(b"/") {
                let mut canon = cwd[..cwd.iter().position(|&b| b == b':').map_or(1, |i| i + 1)].to_vec();
                canon.extend_from_slice(&path);
                canon
            } else {
                let mut canon = cwd.clone();
                if ! canon.ends_with(b"/") {
                    canon.push(b'/');
                }
                canon.extend_from_slice(&path);
                canon
            }
        } else {
            path.to_vec()
        }
    }

    pub fn block(&mut self) -> bool {
        if self.status == Status::Runnable {
            self.status = Status::Blocked;
            true
        } else {
            false
        }
    }

    pub fn unblock(&mut self) -> bool {
        if self.status == Status::Blocked {
            self.status = Status::Runnable;
            true
        } else {
            false
        }
    }

    /// Add a file to the lowest available slot.
    /// Return the file descriptor number or None if no slot was found
    pub fn add_file(&self, file: File) -> Option<usize> {
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
    pub fn remove_file(&self, i: usize) -> Option<File> {
        let mut files = self.files.lock();
        if i < files.len() {
            files[i].take()
        } else {
            None
        }
    }
}
