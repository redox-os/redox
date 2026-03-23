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

use std::fmt::Display;
use std::io;
use std::path::PathBuf;
use std::process::{Command, ExitStatus};

/// Error types used through cookbook.
///
/// When writing IO context, don't use "Failed at XXX". Look at impl Display for suitable word to use.
#[derive(Debug)]
pub enum Error {
    Io {
        source: io::Error,
        path: Option<PathBuf>,
        context: &'static str,
    },
    FileIo {
        source: io::Error,
        src: PathBuf,
        dst: PathBuf,
        context: &'static str,
    },
    Command(Command, ExitStatus),
    Package(pkg::PackageError),
    Pkgar(pkgar::Error),
    Other(String),
}

impl Error {
    pub fn from_last_io_error(context: &'static str) -> Error {
        wrap_io_err!(context)(io::Error::last_os_error())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io {
                source,
                path,
                context,
            } => {
                if let Some(path) = path {
                    write!(f, "{context} failed at \"{}\": {}", path.display(), source)
                } else {
                    write!(f, "{context} failed: {}", source)
                }
            }
            Error::FileIo {
                source,
                src,
                dst,
                context,
            } => {
                write!(
                    f,
                    "{context} failed from \"{}\" to \"{}\": {}",
                    src.display(),
                    dst.display(),
                    source
                )
            }
            Error::Command(command, exit_status) => {
                write!(
                    f,
                    "Failed to run [{:?}]: exited with status {}",
                    command, exit_status
                )
            }
            Error::Package(package_error) => write!(f, "{}", package_error),
            Error::Pkgar(error) => write!(f, "{}", error),
            Error::Other(context) => {
                write!(f, "{context}")
            }
        }
    }
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
    ($src:expr, $dst:expr, $context:expr) => {
        |source| crate::Error::FileIo {
            source,
            src: $src.to_path_buf(),
            dst: $dst.to_path_buf(),
            context: $context,
        }
    };
}

macro_rules! wrap_other_err {
    ($($arg:tt)*) => {
        || crate::Error::Other(format!($($arg)*))
    };
}

macro_rules! bail_other_err {
    ($($arg:tt)*) => {
        return Err(crate::Error::Other(format!($($arg)*)))
    };
}

impl From<&'static str> for Error {
    fn from(value: &'static str) -> Self {
        Error::Other(value.to_string())
    }
}
impl From<String> for Error {
    fn from(value: String) -> Self {
        Error::Other(value)
    }
}

impl From<Error> for String {
    fn from(val: Error) -> Self {
        format!("{}", val)
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

impl From<walkdir::Error> for Error {
    fn from(value: walkdir::Error) -> Self {
        if value.io_error().is_some() {
            let path = value.path().map(|s| s.to_path_buf());
            Error::Io {
                source: value.into_io_error().unwrap(),
                path: path,
                context: "Walkdir error",
            }
        } else {
            wrap_other_err!(
                "Walkdir file system loop found at {:?}",
                value.path().map(|s| s.to_string_lossy().to_string()),
            )()
        }
    }
}

pub(crate) type Result<T> = std::result::Result<T, Error>;

pub(crate) use wrap_io_err;

pub(crate) use wrap_other_err;

pub(crate) use bail_other_err;
