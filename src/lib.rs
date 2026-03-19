pub mod blake3;
pub mod config;
pub mod cook;
pub mod recipe;
pub mod web;

mod progress_bar;

/// Default for maximum number of levels to descend down dependencies tree.
pub const WALK_DEPTH: usize = 16;

/// Default remote package source, for recipes with build type = "remote"
pub const REMOTE_PKG_SOURCE: &str = "https://static.redox-os.org/pkg";

pub fn is_redox() -> bool {
    cfg!(target_os = "redox")
}

// Errors

use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    Io {
        source: io::Error,
        path: Option<PathBuf>,
        context: &'static str,
    },
    Package(pkg::PackageError),
    Pkgar(pkgar::Error),
    Other(String),
}

macro_rules! wrap_io_err {
    ($context:expr) => {
        |source| crate::Error::Io {
            source,
            path: None,
            context: $context,
        }
    };
    ($path:expr, $context:expr) => {
        |source| crate::Error::Io {
            source,
            path: Some($path.to_path_buf()),
            context: $context,
        }
    };
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Error::Other(value)
    }
}

impl From<pkg::PackageError> for Error {
    fn from(value: pkg::PackageError) -> Self {
        Error::Package(value)
    }
}

impl From<pkgar::Error> for Error {
    fn from(value: pkgar::Error) -> Self {
        match value {
            pkgar::Error::Io {
                source,
                path,
                context,
            } => Error::Io {
                source,
                path,
                context,
            },
            _ => Error::Pkgar(value),
        }
    }
}

pub(crate) type Result<T> = std::result::Result<T, Error>;

pub(crate) use wrap_io_err;
