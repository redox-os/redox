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
use context;
use core::fmt::{self, Display, Formatter};
use core::result;
use core::sync::atomic::{AtomicUsize, Ordering};
use spin::{Once, RwLock, RwLockReadGuard, RwLockWriteGuard};

use syscall::error::*;
use syscall::scheme::Scheme;

use self::debug::{DEBUG_SCHEME_ID, DebugScheme};
use self::event::EventScheme;
use self::env::EnvScheme;
use self::initfs::InitFsScheme;
use self::irq::{IRQ_SCHEME_ID, IrqScheme};
use self::null::NullScheme;
use self::pipe::{PIPE_SCHEME_ID, PipeScheme};
use self::root::{ROOT_SCHEME_ID, RootScheme};
use self::sys::SysScheme;
use self::zero::ZeroScheme;

/// Debug scheme
pub mod debug;

/// Kernel events
pub mod event;

/// Environmental variables
pub mod env;

/// InitFS scheme
pub mod initfs;

/// IRQ handling
pub mod irq;

/// Null scheme
pub mod null;

/// Anonymouse pipe
pub mod pipe;

/// Root scheme
pub mod root;

/// System information
pub mod sys;

/// Userspace schemes
pub mod user;

/// Zero scheme
pub mod zero;

/// Limit on number of schemes
pub const SCHEME_MAX_SCHEMES: usize = 65536;

/// Unique identifier for a scheme.
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub struct SchemeId(usize);
impl SchemeId {
    const fn into(self) -> usize {
        self.0
    }
}
impl Display for SchemeId {
    fn fmt(&self, formatter: &mut Formatter) -> result::Result<(), fmt::Error> {
        self.0.fmt(formatter)
    }
}

/// A mutable holder for SchemeId that can safely be shared among threads.
pub struct AtomicSchemeId {
    container: AtomicUsize
}
impl AtomicSchemeId {
    pub const fn new(x: SchemeId) -> Self {
        AtomicSchemeId {
            container: AtomicUsize::new(x.into())
        }
    }
    pub fn load(&self, order: Ordering) -> SchemeId {
        SchemeId(self.container.load(order))
    }
    pub fn store(&self, val: SchemeId, order: Ordering) {
        self.container.store(val.into(), order)
    }
    pub fn swap(&self, val: SchemeId, order: Ordering) -> SchemeId {
        SchemeId(self.container.swap(val.into(), order))
    }
    pub fn compare_and_swap(&self, current: SchemeId, new: SchemeId, order: Ordering) -> SchemeId {
        SchemeId(self.container.compare_and_swap(current.into(), new.into(), order))
    }
    pub fn compare_exchange(&self, current: SchemeId, new: SchemeId, success: Ordering, failure: Ordering) -> result::Result<SchemeId, SchemeId> {
        match self.container.compare_exchange(current.into(), new.into(), success, failure) {
            Ok(result) => Ok(SchemeId(result)),
            Err(result) => Err(SchemeId(result))
        }
    }
    pub fn compare_exchange_weak(&self, current: SchemeId, new: SchemeId, success: Ordering, failure: Ordering) -> result::Result<SchemeId, SchemeId> {
        match self.container.compare_exchange_weak(current.into(), new.into(), success, failure) {
            Ok(result) => Ok(SchemeId(result)),
            Err(result) => Err(SchemeId(result))
        }
    }
}

pub const ATOMIC_SCHEMEID_INIT: AtomicSchemeId = AtomicSchemeId::new(SchemeId(0));

/// Scheme list type
pub struct SchemeList {
    map: BTreeMap<SchemeId, Arc<Box<Scheme + Send + Sync>>>,
    names: BTreeMap<Box<[u8]>, SchemeId>,
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

    pub fn iter(&self) -> ::collections::btree_map::Iter<SchemeId, Arc<Box<Scheme + Send + Sync>>> {
        self.map.iter()
    }

    pub fn iter_name(&self) -> ::collections::btree_map::Iter<Box<[u8]>, SchemeId> {
        self.names.iter()
    }

    /// Get the nth scheme.
    pub fn get(&self, id: SchemeId) -> Option<&Arc<Box<Scheme + Send + Sync>>> {
        self.map.get(&id)
    }

    pub fn get_name(&self, name: &[u8]) -> Option<(SchemeId, &Arc<Box<Scheme + Send + Sync>>)> {
        if let Some(&id) = self.names.get(name) {
            self.get(id).map(|scheme| (id, scheme))
        } else {
            None
        }
    }

    /// Create a new scheme.
    pub fn insert(&mut self, name: Box<[u8]>, scheme: Arc<Box<Scheme + Send + Sync>>) -> Result<SchemeId> {
        if self.names.contains_key(&name) {
            return Err(Error::new(EEXIST));
        }

        if self.next_id >= SCHEME_MAX_SCHEMES {
            self.next_id = 1;
        }

        while self.map.contains_key(&SchemeId(self.next_id)) {
            self.next_id += 1;
        }

        if self.next_id >= SCHEME_MAX_SCHEMES {
            return Err(Error::new(EAGAIN));
        }

        let id = SchemeId(self.next_id);
        self.next_id += 1;

        assert!(self.map.insert(id, scheme).is_none());
        assert!(self.names.insert(name, id).is_none());
        Ok(id)
    }
}

/// Schemes list
static SCHEMES: Once<RwLock<SchemeList>> = Once::new();

/// Initialize schemes, called if needed
fn init_schemes() -> RwLock<SchemeList> {
    let mut list: SchemeList = SchemeList::new();
    ROOT_SCHEME_ID.store(list.insert(Box::new(*b""), Arc::new(Box::new(RootScheme::new()))).expect("failed to insert root scheme"), Ordering::SeqCst);
    DEBUG_SCHEME_ID.store(list.insert(Box::new(*b"debug"), Arc::new(Box::new(DebugScheme))).expect("failed to insert debug scheme"), Ordering::SeqCst);
    list.insert(Box::new(*b"event"), Arc::new(Box::new(EventScheme::new()))).expect("failed to insert event scheme");
    list.insert(Box::new(*b"env"), Arc::new(Box::new(EnvScheme::new()))).expect("failed to insert env scheme");
    list.insert(Box::new(*b"initfs"), Arc::new(Box::new(InitFsScheme::new()))).expect("failed to insert initfs scheme");
    IRQ_SCHEME_ID.store(list.insert(Box::new(*b"irq"), Arc::new(Box::new(IrqScheme))).expect("failed to insert irq scheme"), Ordering::SeqCst);
    list.insert(Box::new(*b"null"), Arc::new(Box::new(NullScheme))).expect("failed to insert null scheme");
    PIPE_SCHEME_ID.store(list.insert(Box::new(*b"pipe"), Arc::new(Box::new(PipeScheme))).expect("failed to insert pipe scheme"), Ordering::SeqCst);
    list.insert(Box::new(*b"sys"), Arc::new(Box::new(SysScheme::new()))).expect("failed to insert sys scheme");
    list.insert(Box::new(*b"zero"), Arc::new(Box::new(ZeroScheme))).expect("failed to insert zero scheme");
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
