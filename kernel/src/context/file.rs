//! File struct

use scheme::SchemeId;

/// A file
//TODO: Close on exec
#[derive(Copy, Clone, Debug)]
pub struct File {
    /// The scheme that this file refers to
    pub scheme: SchemeId,
    /// The number the scheme uses to refer to this file
    pub number: usize,
    /// If events are on, this is the event ID
    pub event: Option<usize>,
}
