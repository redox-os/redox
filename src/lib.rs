pub mod blake3;
pub mod config;
pub mod cook;
pub mod recipe;

mod progress_bar;

/// Default for maximum number of levels to descend down dependencies tree.
pub const WALK_DEPTH: usize = 16;

/// Default remote package source, for recipes with build type = "remote"
pub const REMOTE_PKG_SOURCE: &str = "https://static.redox-os.org/pkg";

pub fn is_redox() -> bool {
    cfg!(target_os = "redox")
}
