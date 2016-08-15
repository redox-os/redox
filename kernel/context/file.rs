//! File struct

/// A file
#[derive(Copy, Clone, Debug)]
pub struct File {
    /// The scheme that this file refers to
    pub scheme: usize,
    /// The number the scheme uses to refer to this file
    pub number: usize,
}
