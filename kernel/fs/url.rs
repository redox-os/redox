use super::Resource;

use alloc::boxed::Box;

use collections::String;
use collections::borrow::ToOwned;

use core::result::Result::{Ok, Err};

use common::slice::GetSlice;

use system::error::{Result, Error};
use system::syscall::{O_CREAT, O_RDWR, O_TRUNC};

/// An URL, see wiki
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Url<'a> {
    scheme: &'a str,
    reference: &'a str,
}

impl<'a> Url<'a> {
    /// Create a new empty URL
    pub fn new() -> Url<'static> {
        Url {
            scheme: "",
            reference: "",
        }
    }

    /// Create from str
    pub fn from_str(string: &'a str) -> Result<Url<'a>> {
        let split = if let Some(x) = string.find(':') {
            x
        } else {
            return Err(Error::new(22));
        };

        Ok(Url {
            scheme: string.get_slice(..split),
            reference: string.get_slice(split + 1..),
        })
    }

    /// Convert the url to string
    pub fn to_string(self) -> String {
        self.scheme.to_owned() + ":" + self.reference
    }

    /// Get the length of this URL
    pub fn len(self) -> usize {
        self.scheme.len() + self.reference.len() + 1
    }

    /// Open this URL (returns a resource)
    pub fn open(self) -> Result<Box<Resource>> {
        ::env().open(self, O_RDWR)
    }

    /// Create this URL (returns a resource)
    pub fn create(self) -> Result<Box<Resource>> {
        ::env().open(self, O_CREAT | O_RDWR | O_TRUNC)
    }

    /// Return the scheme of this url
    pub fn scheme(self) -> &'a str {
        self.scheme
    }

    /// Get the reference (after the ':') of the url
    pub fn reference(self) -> &'a str {
        self.reference
    }

    /// To owned equivalent
    pub fn to_owned(&self) -> OwnedUrl {
        OwnedUrl {
            scheme: self.scheme.to_owned(),
            reference: self.reference.to_owned(),
        }
    }

    /// Into a cow
    pub fn to_cow(self) -> CowUrl<'a> {
        CowUrl::Ref(self)
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct OwnedUrl {
    scheme: String,
    reference: String,
}

impl OwnedUrl {
    /// Create a new empty URL
    pub fn new() -> OwnedUrl {
        OwnedUrl {
            scheme: String::new(),
            reference: String::new(),
        }
    }

    /// Into a cow
    pub fn into_cow<'a>(self) -> CowUrl<'a> {
        /*
             ______________
            < Mooooooooooo >
             --------------
                    \   ^__^
                     \  (oo)\_______
                        (__)\       )\/\
                            ||----w |
                            ||     ||

         */
        CowUrl::Owned(self)
    }

    /// As an unowned URL
    pub fn as_url(&self) -> Url {
        Url {
            scheme: &self.scheme,
            reference: &self.reference,
        }
    }
}

/// A Copy-On-Write URL
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum CowUrl<'a> {
    Ref(Url<'a>),
    Owned(OwnedUrl),
}

impl<'a> CowUrl<'a> {
    /// As URL
    pub fn as_url(&self) -> Url {
        match self {
            &CowUrl::Ref(u) => u,
            &CowUrl::Owned(ref u) => u.as_url(),
        }
    }
}
