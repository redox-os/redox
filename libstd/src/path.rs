use fmt;
use mem;
use core_collections::borrow::{Cow, IntoCow};
use string::String;

pub struct Display<'a> {
    string: &'a str
}

impl<'a> fmt::Display for Display<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.string)
    }
}

pub struct Path {
    pub inner: str,
}

impl Path {
    /// Create a new path
    /// # Safety
    /// This uses the same logic in libstd, it should be safe for valid &str
    pub fn new<S: AsRef<str> + ?Sized>(s: &S) -> &Path {
        unsafe { mem::transmute(s.as_ref()) }
    }
}

impl AsRef<Path> for str {
    fn as_ref(&self) -> &Path {
        Path::new(self)
    }
}

impl AsRef<Path> for String {
    fn as_ref(&self) -> &Path {
        Path::new(self)
    }
}

impl AsRef<Path> for PathBuf {
    fn as_ref(&self) -> &Path {
        Path::new(&self.inner)
    }
}

#[derive(Debug)]
pub struct PathBuf {
    pub inner: String,
}

impl From<String> for PathBuf {
    fn from(inner: String) -> PathBuf {
        PathBuf { inner: inner }
    }
}

impl PathBuf {
    pub fn to_str(&self) -> Option<&str> {
        Some(&self.inner)
    }

    pub fn to_string_lossy(&self) -> Cow<str> {
        self.inner.clone().into_cow()
    }

    pub fn to_string(&self) -> String {
        self.inner.clone()
    }

    pub fn display(&self) -> Display {
        Display {
            string: &self.inner
        }
    }
}
