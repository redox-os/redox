use alloc::boxed::Box;

use collections::{BTreeMap, String};

use fs::{KScheme, Resource, Url, VecResource};

use system::error::{ENOENT, Error, Result};

#[path="../../build/initfs.gen"]
pub mod gen;

/// A memory scheme
pub struct InitFsScheme {
    pub files: BTreeMap<&'static str, &'static [u8]>,
}

impl InitFsScheme {
    pub fn new() -> Box<InitFsScheme> {
        Box::new(InitFsScheme { files: gen::gen() })
    }
}

impl KScheme for InitFsScheme {
    fn scheme(&self) -> &str {
        "initfs"
    }

    fn open(&mut self, url: Url, _: usize) -> Result<Box<Resource>> {
        let reference = url.reference().trim_matches('/');
        if reference.is_empty() {
            let mut list = String::new();

            for file in self.files.iter() {
                if !list.is_empty() {
                    list.push('\n');
                }
                list.push_str(file.0);
            }

            Ok(box VecResource::new(url.to_string(), list.into_bytes()))
        } else {
            if let Some(data) = self.files.get(reference) {
                Ok(box VecResource::new(url.to_string(), data.to_vec()))
            } else {
                Err(Error::new(ENOENT))
            }
        }
    }
}
