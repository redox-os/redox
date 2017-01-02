use alloc::arc::Arc;
use alloc::boxed::Box;
use collections::{BTreeMap, Vec, VecDeque};
use spin::Mutex;

use arch;
use context::file::File;
use context::memory::{Grant, Memory, SharedMemory, Tls};
use scheme::{SchemeNamespace, FileHandle};
use syscall::data::Event;
use sync::{WaitMap, WaitQueue};

/// Unique identifier for a context (i.e. `pid`).
use ::core::sync::atomic::AtomicUsize;
int_like!(ContextId, AtomicContextId, usize, AtomicUsize);

/// The status of a context - used for scheduling
/// See syscall::process::waitpid and the sync module for examples of usage
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
    pub id: ContextId,
    /// The ID of the parent context
    pub ppid: ContextId,
    /// The real user id
    pub ruid: u32,
    /// The real group id
    pub rgid: u32,
    /// The real namespace id
    pub rns: SchemeNamespace,
    /// The effective user id
    pub euid: u32,
    /// The effective group id
    pub egid: u32,
    /// The effective namespace id
    pub ens: SchemeNamespace,
    /// Status of context
    pub status: Status,
    /// Context running or not
    pub running: bool,
    /// CPU ID, if locked
    pub cpu_id: Option<usize>,
    /// Context is halting parent
    pub vfork: bool,
    /// Context is being waited on
    pub waitpid: Arc<WaitMap<ContextId, usize>>,
    /// Context should handle pending signals
    pub pending: VecDeque<u8>,
    /// Context should wake up at specified time
    pub wake: Option<(u64, u64)>,
    /// The architecture specific context
    pub arch: arch::context::Context,
    /// Kernel FX - used to store SIMD and FPU registers on context switch
    pub kfx: Option<Box<[u8]>>,
    /// Kernel stack
    pub kstack: Option<Box<[u8]>>,
    /// Executable image
    pub image: Vec<SharedMemory>,
    /// User heap
    pub heap: Option<SharedMemory>,
    /// User stack
    pub stack: Option<Memory>,
    /// User Thread local storage
    pub tls: Option<Tls>,
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
    pub files: Arc<Mutex<Vec<Option<File>>>>,
    /// Files exported through `dup_export`.
    pub exported_files: Mutex<FileExportMap>,
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Debug)]
pub struct FileExport {
    pub target: ContextId,
    key: Box<[u8]>,
}
type FileExportMap = BTreeMap<FileExport, (FileHandle, File)>;

impl Context {
    pub fn new(id: ContextId) -> Context {
        Context {
            id: id,
            ppid: ContextId::from(0),
            ruid: 0,
            rgid: 0,
            rns: SchemeNamespace::from(0),
            euid: 0,
            egid: 0,
            ens: SchemeNamespace::from(0),
            status: Status::Blocked,
            running: false,
            cpu_id: None,
            vfork: false,
            waitpid: Arc::new(WaitMap::new()),
            pending: VecDeque::new(),
            wake: None,
            arch: arch::context::Context::new(),
            kfx: None,
            kstack: None,
            image: Vec::new(),
            heap: None,
            stack: None,
            tls: None,
            grants: Arc::new(Mutex::new(Vec::new())),
            name: Arc::new(Mutex::new(Vec::new())),
            cwd: Arc::new(Mutex::new(Vec::new())),
            events: Arc::new(WaitQueue::new()),
            env: Arc::new(Mutex::new(BTreeMap::new())),
            files: Arc::new(Mutex::new(Vec::new())),
            exported_files: Mutex::new(BTreeMap::new()),
        }
    }

    /// Make a relative path absolute
    /// Given a cwd of "scheme:/path"
    /// This function will turn "foo" into "scheme:/path/foo"
    /// "/foo" will turn into "scheme:/foo"
    /// "bar:/foo" will be used directly, as it is already absolute
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

    /// Block the context, and return true if it was runnable before being blocked
    pub fn block(&mut self) -> bool {
        if self.status == Status::Runnable {
            self.status = Status::Blocked;
            true
        } else {
            false
        }
    }

    /// Unblock context, and return true if it was blocked before being marked runnable
    pub fn unblock(&mut self) -> bool {
        if self.status == Status::Blocked {
            self.status = Status::Runnable;
            if let Some(cpu_id) = self.cpu_id {
                if cpu_id != ::cpu_id() {
                    // Send IPI if not on current CPU
                    // TODO: Make this more architecture independent
                    unsafe { arch::device::local_apic::LOCAL_APIC.ipi(cpu_id) };
                }
            }
            true
        } else {
            false
        }
    }

    /// Add a file to the lowest available slot.
    /// Return the file descriptor number or None if no slot was found
    pub fn add_file(&self, file: File) -> Option<FileHandle> {
        let mut files = self.files.lock();
        for (i, mut file_option) in files.iter_mut().enumerate() {
            if file_option.is_none() {
                *file_option = Some(file);
                return Some(FileHandle::from(i));
            }
        }
        let len = files.len();
        if len < super::CONTEXT_MAX_FILES {
            files.push(Some(file));
            Some(FileHandle::from(len))
        } else {
            None
        }
    }

    /// Get a file
    pub fn get_file(&self, i: FileHandle) -> Option<File> {
        let files = self.files.lock();
        if i.into() < files.len() {
            files[i.into()]
        } else {
            None
        }
    }

    /// Insert a file with a specific handle number. This is used by dup2
    /// Return the file descriptor number or None if the slot was not empty, or i was invalid
    pub fn insert_file(&self, i: FileHandle, file: File) -> Option<FileHandle> {
        let mut files = self.files.lock();
        if i.into() < super::CONTEXT_MAX_FILES {
            while i.into() >= files.len() {
                files.push(None);
            }
            if files[i.into()].is_none() {
                files[i.into()] = Some(file);
                Some(i)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Remove a file
    // TODO: adjust files vector to smaller size if possible
    pub fn remove_file(&self, i: FileHandle) -> Option<File> {
        let mut files = self.files.lock();
        if i.into() < files.len() {
            files[i.into()].take()
        } else {
            None
        }
    }

    pub fn export_file(&self, target: ContextId, key: &[u8], new_id_for_scheme: FileHandle, file: File) -> Option<(FileHandle, File)> {
        use collections::btree_map::Entry::*;
        let mut lock = self.exported_files.lock();
        let entry = lock.entry(FileExport {
            key: key.to_vec().into_boxed_slice(),
            target: target,
        });
        match entry {
            Vacant(vacant) => {
                vacant.insert((new_id_for_scheme, file));
                None
            }
            Occupied(mut occupied) => {
                Some(occupied.insert((new_id_for_scheme, file)))
            }
        }
    }

    pub fn pop_exported_file(&self, target: ContextId, key: &[u8]) -> Option<(FileHandle, File)> {
        let mut lock = self.exported_files.lock();
        lock.remove(&FileExport {
            key: key.to_vec().into_boxed_slice(),
            target: target,
        })
    }
}
