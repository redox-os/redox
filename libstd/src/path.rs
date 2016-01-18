use string::String;

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
}
