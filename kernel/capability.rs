//! Implementation of capabilities.
//!
//! Capabilities are the primitive Redox uses for privilege control.
//!
//! A process that _has_ a capability, can make use of some features. A
//! process that doesn't have a capability cannot. Note that features can
//! be implemented by the kernel, but can just as well be implemented by
//! any userspace process. Indeed, any process can dynamically define new
//! capabilities and decide to grant them to other processes.

use alloc::boxed::Box;
use collections::BTreeMap;

/// A capability.
///
/// In Redox, capabilities are a scheme-controlled byte sequence, and the actual semantics and
/// meaning is left to the scheme server.
///
/// # Subcapabilities
///
/// Every capability is said to have a "kind", which defines how it can be passed between processes
/// or contexts. If the kind of capability X "implies" (i.e. is stronger or equal to) the kind of
/// capability Y, then Y is said to be a subcapability of X.
pub struct Capability {
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
    data: Box<[u8]>,

    /// Definition of the dynamic copy/send semantics of this capability.
    kind: Kind,
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

    /// Set the capability data (byte sequence).
    ///
    /// You ought to be extremely careful with this. The user shouldn't be able to arbitrarily
    /// control the capability data as this means the user is able to give themself arbitrary
    /// powers.
    pub fn set_data(&mut self, data: Box<[u8]>) {
        self.data = data;
    }
    // FIXME: Why would we need to modify the capability data at all?
}

/// A capability kind.
///
/// This defines the semantics of passing, copying, transfering, and sending capabilities across
/// contexts or processes.
#[derive(PartialEq, Eq, Ord, PartialOrd, Clone, Copy, Debug)]
enum Kind {
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

/// A set of capabilities.
#[derive(Debug)]
pub struct CapabilitySet {
    /// The capability sequence to kind map.
    capabilities: BTreeMap<Box<[u8]>, Kind>,
}

impl CapabilitySet {
    /// Insert a capability into the set.
    pub fn insert(&mut self, capability: Capability) -> &mut CapabilitySet {
        self.capabilities.insert(capability.data, capability.kind);

        self
    }

    /// Check if `self` is subset (or equal to) `other`.
    ///
    /// This is useful for determining if you can downgrade `other` to `self` and pass it on.
    ///
    /// If `other` contains every capability or subcapability of elements in `self`, `self` is said
    /// to be a subset of `other`.
    pub fn subset_of(&self, other: &CapabilitySet) -> bool {
        // Iterate over the map of data to kinds and searching .
        for (data, kind) in &other.capabilities {
            if !self.contains_imp(data, *kind) {
                // The lhs didn't contain the element, hence it cannot be a subset.
                return false;
            }
        }

        // Everything matched. It's a subset!
        true
    }

    /// Does this set contains this cability or a subcapability of it?
    pub fn contains(&self, elem: &Capability) -> bool {
        self.contains_imp(&elem.data, elem.kind)
    }

    /// Internal `contains` method.
    fn contains_imp(&self, data: &[u8], kind: Kind) -> bool {
        if let Some(lhs_kind) = self.capabilities.get(data) {
            // The kind of the capability must be implied.
            *lhs_kind <= kind
        } else {
            // The lhs did not contains the capability data in question.
            false
        }
    }
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
