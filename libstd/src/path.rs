use fmt;
use mem;
use core_collections::borrow::{Cow, IntoCow, ToOwned};
use string::String;

pub struct Display<'a> {
    string: &'a str
}

impl<'a> fmt::Display for Display<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.string)
    }
}

pub enum Component<'a> {
    Prefix(&'a str),
    RootDir,
    CurDir,
    ParentDir,
    Normal(&'a str),
}

/// A slice of a path (akin to `str`).
///
/// This type supports a number of operations for inspecting a path, including
/// breaking the path into its components (separated by `/` or `\`, depending on
/// the platform), extracting the file name, determining whether the path is
/// absolute, and so on. More details about the overall approach can be found in
/// the module documentation.
///
/// This is an *unsized* type, meaning that it must always be used behind a
/// pointer like `&` or `Box`.
///
/// # Examples
///
/// ```
/// use std::path::Path;
///
/// let path = Path::new("/tmp/foo/bar.txt");
/// let file = path.file_name();
/// let extension = path.extension();
/// let parent_dir = path.parent();
/// ```
///
#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
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

    pub fn to_path_buf(&self) -> PathBuf {
        PathBuf::from(self.inner.to_owned())
    }

    pub fn to_str(&self) -> Option<&str> {
        Some(&self.inner)
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

impl AsRef<Path> for Path {
    fn as_ref(&self) -> &Path {
        &self
    }
}

impl AsRef<Path> for PathBuf {
    fn as_ref(&self) -> &Path {
        Path::new(&self.inner)
    }
}

impl<'a> From<&'a str> for &'a Path {
    fn from(inner: &'a str) -> &'a Path {
        unsafe { mem::transmute(inner) }
    }
}

/// An owned, mutable path (akin to `String`).
///
/// This type provides methods like `push` and `set_extension` that mutate the
/// path in place. It also implements `Deref` to `Path`, meaning that all
/// methods on `Path` slices are available on `PathBuf` values as well.
///
/// More details about the overall approach can be found in
/// the module documentation.
///
/// # Examples
///
/// ```
/// use std::path::PathBuf;
///
/// let mut path = PathBuf::from("c:\\");
/// path.push("windows");
/// path.push("system32");
/// path.set_extension("dll");
/// ```
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PathBuf {
    pub inner: String,
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

    pub fn file_name(&self) -> Option<&str> {
        self.inner.split(is_separator).last()
    }
}

impl<'a> From<&'a str> for PathBuf {
    fn from(inner: &'a str) -> PathBuf {
        PathBuf { inner: inner.to_owned() }
    }
}

impl From<String> for PathBuf {
    fn from(inner: String) -> PathBuf {
        PathBuf { inner: inner }
    }
}

pub fn is_separator(c: char) -> bool {
    c == '/' || c == ':'
}
