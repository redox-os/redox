use fmt;
use string::String;

pub struct Display<'a> {
    string: &'a str
}

impl<'a> fmt::Display for Display<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.string)
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

    pub fn to_string(&self) -> String {
        self.inner.clone()
    }

    pub fn display(&self) -> Display {
        Display {
            string: &self.inner
        }
    }
}
