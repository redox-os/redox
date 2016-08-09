use alloc::boxed::Box;

use collections::{BTreeMap, String};

use core::cmp::{min, max};

use fs::{KScheme, Resource, ResourceSeek, VecResource};

use system::error::{Error, Result, ENOENT};
use system::syscall::{MODE_DIR, MODE_FILE, Stat};

#[path="../../build/initfs.gen"]
pub mod gen;

/// Init Filesystem resource
pub struct InitFsResource {
    path: String,
    data: &'static [u8],
    seek: usize,
}

impl InitFsResource {
    pub fn new(path: String, data: &'static [u8]) -> Self {
        InitFsResource {
            path: path,
            data: data,
            seek: 0,
        }
    }
}

impl Resource for InitFsResource {
    fn dup(&self) -> Result<Box<Resource>> {
        Ok(box InitFsResource {
            path: self.path.clone(),
            data: self.data,
            seek: self.seek,
        })
    }

    fn path(&self, buf: &mut [u8]) -> Result <usize> {
        let path = self.path.as_bytes();

        let mut i = 0;
        while i < buf.len() && i < path.len() {
            buf[i] = path[i];
            i += 1;
        }

        Ok(i)
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut i = 0;
        while i < buf.len() && self.seek < self.data.len() {
            match self.data.get(self.seek) {
                Some(b) => buf[i] = *b,
                None => (),
            }
            self.seek += 1;
            i += 1;
        }
        return Ok(i);
    }

    fn seek(&mut self, pos: ResourceSeek) -> Result<usize> {
        match pos {
            ResourceSeek::Start(offset) => self.seek = min(self.data.len(), offset),
            ResourceSeek::Current(offset) =>
                self.seek = max(0, min(self.seek as isize, self.seek as isize + offset)) as usize,
            ResourceSeek::End(offset) =>
                self.seek = max(0,
                                min(self.seek as isize,
                                    self.data.len() as isize +
                                    offset)) as usize,
        }
        return Ok(self.seek);
    }

    fn stat(&self, stat: &mut Stat) -> Result<()> {
        stat.st_size = self.data.len() as u32;
        stat.st_mode = MODE_FILE;
        Ok(())
    }

    fn sync(&mut self) -> Result<()> {
        Ok(())
    }
}

/// A memory scheme
pub struct InitFsScheme {
    pub files: BTreeMap<&'static str, &'static [u8]>
}

impl InitFsScheme {
    pub fn new() -> Box<InitFsScheme> {
        Box::new(InitFsScheme {
            files: gen::gen()
        })
    }
}

impl KScheme for InitFsScheme {
    fn scheme(&self) -> &str {
        "initfs"
    }

    fn open(&mut self, url: &str, _: usize) -> Result<Box<Resource>> {
        let reference = url.splitn(2, ":").nth(1).unwrap_or("").trim_matches('/');

        if let Some(data) = self.files.get(reference) {
            Ok(box InitFsResource::new(format!("initfs:/{}", reference), data))
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
                    Ok(box VecResource::new(format!("initfs:/{}/", reference), list.into_bytes(), MODE_DIR))
                } else {
                    Ok(box VecResource::new(format!("initfs:/"), list.into_bytes(), MODE_DIR))
                }
            } else {
                Err(Error::new(ENOENT))
            }
        }
    }
}
