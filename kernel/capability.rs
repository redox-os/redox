//! Implementation of capabilities.
//!
//! Capabilities are the primitive Redox uses for privilege control.
//!
//! A process that _has_ a capability, can make use of some features. A
//! process that doesn't have a capability cannot. Note that features can
//! be implemented by the kernel, but can just as well be implemented by
//! any userspace process. Indeed, any process can dynamically define new
//! capabilities and decide to grant them to other processes.

use alloc::arc::Arc;
use alloc::boxed::Box;
use collections::{ BTreeMap, Vec };
use core::ops::Deref;

#[derive(PartialEq, Eq, Debug)]
pub struct Data(Box<[u8]>);
impl Data {
    pub fn new(data: Box<[u8]>) -> Self {
        Data(data)
    }
}
impl Deref for Data {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl Capability {
    /// Is this capability inheritable (i.e. possible to pass to children)?
    pub fn is_inheritable(&self) -> bool {
        self.kind >= Kind::Inherit
    }

    /// Is this capability sendable (i.e. possible to pass to arbitrary processes)?
    pub fn is_sendable(&self) -> bool{
        self.kind >= Kind::Send
    }
}

/// A capability kind.
///
/// This defines the semantics of passing, copying, transfering, and sending capabilities across
/// contexts or processes.
#[derive(PartialEq, Eq, Ord, PartialOrd, Clone, Copy, Debug)]
pub enum Kind {
    /// A static capability.
    ///
    /// This means that you cannot pass it on to other processes. It will always stay in the
    /// process.
    Static = 0,
    /// An inheritable capability.
    ///
    /// This means that I can pass it to child processes, but not arbitrary proceses. Kind of like
    /// how Roman Law allows fathers to kill or maim their own children but not other people's
    /// children.
    Inherit = 1,
    /// A sendable capability.
    ///
    /// This means that I can pass the capability to any arbitrary process. Note that this can be
    /// exploited in unfortunate ways and should be used very carefully. In particular, a malicious
    /// process could givethe capability to every process on the system and thus weakening
    /// security.
    Send = 2,
}
impl Kind {
    pub fn from_usize(val: usize) -> Option<Kind> {
        match val {
            0 => Some(Kind::Static),
            1 => Some(Kind::Inherit),
            2 => Some(Kind::Send),
            _ => None
        }
    }
}


/// Representation of a capability.
///
/// In Redox, capabilities are a scheme-controlled byte sequence, and the actual semantics and
/// meaning is left to the scheme server.
///
/// A capability can be uniquely identified either by either of:
/// - its properties `owner`, `owner_index`
/// - its definition `scheme`, `data`
///
/// # Subcapabilities
///
/// Every capability is said to have a "kind", which defines how it can be passed between processes
/// or contexts. If the kind of capability X "implies" (i.e. is stronger or equal to) the kind of
/// capability Y, then Y is said to be a subcapability of X.

#[derive(Debug)]
pub struct Capability {
    /// Identification of the process that first issued this capability.
    ///
    /// When `owner` dies, all its `Capability` are revoked, recursively.
    pub owner: usize,

    /// The index of the capability among the owner's capabilities.
    pub owner_index: usize,

    /// Identification of the scheme to which this capability is dedicated.
    ///
    /// This guarantees that `owner` implements `scheme`.
    pub scheme: Box<[u8]>,

    /// The inner data.
    ///
    /// Interpretation of this data is left to the scheme that defined it.
    ///
    /// For instance, a scheme `fs` could distribute a capability `+rwx/some/file`,
    /// a capability `+rw/some/other/file`, etc. to selectively allow processes to
    /// access individual files or directory.
    ///
    /// When a process `P` decides to perform an operation on `fs:some/path`, the
    /// implementation of `fs` can ask the kernel for the list of `fs` capabilities
    /// owned by `P`. Based on this list, the implementation of `fs` will decide whether
    /// to let `P` perform this operation.
    pub data: Data,

    /// Bounds on the dynamic copy/send semantics of this capability.
    pub kind: Kind,
}

#[derive(Debug)]
pub struct CapabilitySet {
    /// Mapping from local handle to capability.
    by_local_handle: Vec<Option<Instance>>,
    // FIXME: We probably need other tables to speed up operations.
    // FIXME: Instead of a Vec<Option<Cap>>, this should be a free-list.
}
impl CapabilitySet {
    pub fn new() -> Self {
        CapabilitySet {
            by_local_handle: vec![] // FIXME: Default size?
        }
    }

    /// If a capability is already part of this `CapabilitySet`, get its index.
    pub fn get(&self, scheme: &[u8], data: &[u8]) -> Option<(usize, &Instance)> {
        for (i, cap) in self.by_local_handle.iter().enumerate() {
            if let &Some(ref cap) = cap {
                debug_assert!(cap.local_rc > 0);
                debug_assert!(Arc::strong_count(&cap.root) > 0);
                if &*cap.root.scheme == scheme && *cap.root.data.0 == *data {
                    return Some((i, cap))
                }
            }
        }
        None
    }
    pub fn alloc(&mut self) -> (usize, &mut Option<Instance>) {
        let mut slot = None;
        for (i, cap) in self.by_local_handle.iter().enumerate() {
            if cap.is_none() {
                slot = Some(i);
                break;
            }
        }
        let index = match slot {
            Some(index) => index,
            None => {
                self.by_local_handle.push(None);
                self.by_local_handle.len() - 1
            }
        };
        (index, &mut self.by_local_handle[index])
    }
}

#[derive(Debug)]
pub struct Instance {
    /// The number of times this capability has been granted to this process by distinct
    /// processes.
    /// Once this number decreases to 0, we remove the capability from `by_local_handle`.
    pub local_rc: usize,

    /// Bounds on the dynamic copy/send semantics of this instance of the capability.
    /// Invariant: self.kind <= self.root.kind
    pub kind: Kind,

//    sent_to: HashSet<usize /* pid */>, // FIXME: Implement.

    /// A globally unique representation of this capability.
    /// Once the refcount is down to 1, we need to remove the capability from its owner.
    pub root: Arc<Capability>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_poset() {
        assert!(Kind::Static < Kind::Inherit);
        assert!(Kind::Static < Kind::Send);
        assert!(Kind::Inherit < Kind::Send);
    }
}
