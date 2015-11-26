pub use self::intex::Intex;
pub use self::mutex::Mutex;
pub use self::recursive_mutex::RecursiveMutex;
pub use self::rwlock::RwLock;

/// Interrupt exclution - use carefully
pub mod intex;
/// Mutual exclusion - use with caution
pub mod mutex;
/// Recursive mutual exclusion - use with even more caution
pub mod recursive_mutex;
/// Readers-writer lock - use with caution
pub mod rwlock;
