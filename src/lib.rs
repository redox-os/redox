pub mod blake3;
pub mod recipe;

mod progress_bar;

/// Default for maximum number of levels to descend down dependencies tree.
pub const WALK_DEPTH: usize = 16;

#[cfg(target_os = "redox")]
pub fn is_redox() -> bool {
    true
}

#[cfg(not(target_os = "redox"))]
pub fn is_redox() -> bool {
    false
}
