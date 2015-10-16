use redox::Box;
use redox::fs::file::File;
use redox::mem;
use redox::rand;
use redox::{str, String, ToString};
use redox::Vec;

/// Pseudo-randomness Scheme
pub struct Scheme;

impl Scheme {
    pub fn scheme(&self) -> Box<Self> {
        box Scheme
    }

    pub fn open(&mut self, url: &str) -> Option<Box<Resource>> {
        Some(box Resource::new(File::open("random://"), String::from_num(rand()).to_utf8()))
    }
}
