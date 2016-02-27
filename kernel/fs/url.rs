use super::Resource;

use alloc::boxed::Box;

use collections::String;
use collections::string::ToString;

use common::slice::GetSlice;

use system::error::Result;
use system::syscall::{O_CREAT, O_RDWR, O_TRUNC};

/// A URL, see wiki
pub struct Url {
    pub string: String,
}

impl Url {
    /// Create a new empty URL
    pub fn new() -> Url {
        Url {
            string: String::new()
        }
    }

    /// Create from str
    pub fn from_str(string: &str) -> Url {
        Url {
            string: string.to_string()
        }
    }

    /// Create from string
    pub fn from_string(string: String) -> Url {
        Url {
            string: string
        }
    }

    /// Get the length of this URL
    pub fn len(&self) -> usize {
        self.string.len()
    }

    /// Open this URL (returns a resource)
    pub fn open(&self) -> Result<Box<Resource>> {
        ::env().open(&self, O_RDWR)
    }

    /// Create this URL (returns a resource)
    pub fn create(&self) -> Result<Box<Resource>> {
        ::env().open(&self, O_CREAT | O_RDWR | O_TRUNC)
    }

    /// Return the scheme of this url
    pub fn scheme(&self) -> &str {
        self.string.get_slice(..self.string.find(':'))
    }

    /// Get the reference (after the ':') of the url
    pub fn reference(&self) -> &str {
        self.string.get_slice(self.string.find(':').map(|a| a + 1)..)
    }
}

impl Clone for Url {
    fn clone(&self) -> Url {
        Url {
            string: self.string.clone()
        }
    }
}
