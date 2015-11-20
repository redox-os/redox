/*
 * The following states are written to disk as part of the normal
 * SPA lifecycle: Active, Exported, Destroyed, Spare, L2Cache.  The remaining
 * states are software abstractions used at various levels to communicate
 * pool state.
 */
pub enum PoolState {
    Active = 0,       // In active use
    Exported,         // Explicitly exported
    Destroyed,        // Explicitly destroyed
    Spare,            // Reserved for hot spare use
    L2Cache,          // Level 2 ARC device
    Uninitialized,    // Internal spa_t state
    Unavailable,      // Internal libzfs state
    PotentiallyActive // Internal libzfs state
}
