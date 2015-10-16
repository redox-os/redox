use redox::Box;
use redox::fs::file::File;
use redox::mem;
use redox::{str, String, ToString};
use redox::Vec;

/// A memory scheme
pub struct Scheme;

impl Scheme {
    fn scheme(&self) -> Box<Self> {
        box Scheme
    }

    fn open(&mut self, url: &URL) -> Option<Box<Resource>> {
        let string = "Memory Used: ".to_string() + memory::memory_used() / 1024 + " KB\n" +
                     "Memory Free: " + memory::memory_free() / 1024 + " KB";
        Some(box Resource::new(File::open("memory://"), string.to_utf8()))
    }
}
