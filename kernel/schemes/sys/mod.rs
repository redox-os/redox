use alloc::boxed::Box;

use collections::{BTreeMap, String};

use fs::{KScheme, Resource, VecResource};

use system::error::{Error, ENOENT, Result};
use system::syscall::MODE_DIR;

mod context;
mod interrupt;
mod log;
mod memory;
mod test;

/// System information scheme
pub struct SysScheme {
    pub files: BTreeMap<&'static str, Box<Fn() -> Result<Box<Resource>>>>
}

impl SysScheme {
    pub fn new() -> Box<SysScheme> {
        let mut files: BTreeMap<&'static str, Box<Fn() -> Result<Box<Resource>>>> = BTreeMap::new();

        files.insert("context", box move || context::resource());
        files.insert("interrupt", box move || interrupt::resource());
        files.insert("log", box move || log::resource());
        files.insert("memory", box move || memory::resource());
        files.insert("test", box move || test::resource());

        Box::new(SysScheme {
            files: files
        })
    }
}

impl KScheme for SysScheme {
    fn scheme(&self) -> &str {
        "sys"
    }

    fn open(&mut self, url: &str, _: usize) -> Result<Box<Resource>> {
        let reference = url.splitn(2, ":").nth(1).unwrap_or("").trim_matches('/');

        if let Some(func) = self.files.get(reference) {
            func()
        } else {
            let mut list = String::new();

            'files: for file in self.files.iter() {
                let mut file_parts = file.0.split('/');

                if ! reference.is_empty() {
                    let mut ref_parts = reference.split('/');

                    while let Some(ref_part) = ref_parts.next() {
                        if let Some(file_part) = file_parts.next() {
                            if file_part != ref_part {
                                continue 'files;
                            }
                        } else {
                            continue 'files;
                        }
                    }
                }

                if let Some(file_part) = file_parts.next() {
                    for item in list.split('\n') {
                        if item == file_part {
                            continue 'files;
                        }
                    }
                    if ! list.is_empty() {
                        list.push('\n');
                    }
                    list.push_str(file_part);
                }
            }

            if ! list.is_empty() {
                if ! reference.is_empty() {
                    Ok(box VecResource::new(format!("sys:/{}/", reference), list.into_bytes(), MODE_DIR))
                } else {
                    Ok(box VecResource::new(format!("sys:/"), list.into_bytes(), MODE_DIR))
                }
            } else {
                Err(Error::new(ENOENT))
            }
        }
    }
}
