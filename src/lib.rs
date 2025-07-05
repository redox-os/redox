pub mod blake3;
pub mod recipe;

mod progress_bar;

/// Default for maximum number of levels to descend down dependencies tree.
pub const WALK_DEPTH: usize = 16;
