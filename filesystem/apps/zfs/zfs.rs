use std::result;

/// The error type used throughout ZFS
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Error {
    NoEntity,
    Invalid,
}

/// The Result type used throughout ZFS
pub type Result<T> = result::Result<T, Error>;

/// The following states are written to disk as part of the normal
/// SPA lifecycle: Active, Exported, Destroyed, Spare, L2Cache.  The remaining
/// states are software abstractions used at various levels to communicate
/// pool state.
#[derive(Copy, Clone, PartialEq)]
pub enum PoolState {
    Active = 0, // In active use
    Exported, // Explicitly exported
    Destroyed, // Explicitly destroyed
    Spare, // Reserved for hot spare use
    L2Cache, // Level 2 ARC device
    Uninitialized, // Internal spa_t state
    Unavailable, // Internal libzfs state
    PotentiallyActive, // Internal libzfs state
}

/// Internal SPA load state.  Used by FMA diagnosis engine.
#[derive(Copy, Clone, PartialEq)]
pub enum SpaLoadState {
    None, // no load in progress
    Open, // normal open
    Import, // import in progress
    TryImport, // tryimport in progress
    Recover, // recovery requested
    Error, // load failed
}
