//! Typestrong address segregation.

/// A physical address in memory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Physical {
    /// The position.
    ///
    /// Note that we do not use a pointer here to avoid simple mistakes where the programmer
    /// confuse virtual and physical.
    pub inner: u64,
}
