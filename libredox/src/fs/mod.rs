use core::usize;

use io::{Read, Write, Seek, SeekFrom};
use str;
use string::{String, ToString};
use vec::Vec;

use syscall::{sys_open, sys_dup, sys_close, sys_execve, sys_fpath, sys_ftruncate, sys_read,
              sys_write, sys_lseek, sys_fsync, sys_chdir, sys_mkdir};
use syscall::common::{O_RDWR, O_CREAT, O_TRUNC, SEEK_SET, SEEK_CUR, SEEK_END};

/// A Unix-style file
pub struct File {
    /// The id for the file
    fd: usize,
}

impl File {
    pub fn exec(path: &str, args: &[&str]) -> bool {
        let path_c = path.to_string() + "\0";

        let mut args_vec: Vec<String> = Vec::new();
        for arg in args.iter() {
            args_vec.push(arg.to_string() + "\0");
        }

        let mut args_c: Vec<*const u8> = Vec::new();
        for arg_vec in args_vec.iter() {
            args_c.push(arg_vec.as_ptr());
        }
        args_c.push(0 as *const u8);

        unsafe { sys_execve(path_c.as_ptr(), args_c.as_ptr()) == 0 }
    }

    /// Open a new file using a path
    pub fn open(path: &str) -> Option<File> {
        unsafe {
            let fd = sys_open((path.to_string() + "\0").as_ptr(), O_RDWR, 0);
            if fd == usize::MAX {
                None
            } else {
                Some(File { fd: fd })
            }
        }
    }

    /// Create a new file using a path
    pub fn create(path: &str) -> Option<File> {
        unsafe {
            let fd = sys_open((path.to_string() + "\0").as_ptr(),
                              O_CREAT | O_RDWR | O_TRUNC,
                              0);
            if fd == usize::MAX {
                None
            } else {
                Some(File { fd: fd })
            }
        }
    }

    /// Duplicate the file
    pub fn dup(&self) -> Option<File> {
        unsafe {
            let new_fd = sys_dup(self.fd);
            if new_fd == usize::MAX {
                None
            } else {
                Some(File { fd: new_fd })
            }
        }
    }

    /// Get the canonical path of the file
    pub fn path(&self) -> Option<String> {
        unsafe {
            let mut buf: [u8; 4096] = [0; 4096];
            let count = sys_fpath(self.fd, buf.as_mut_ptr(), buf.len());
            if count == usize::MAX {
                None
            } else {
                Some(String::from_utf8_unchecked(Vec::from(&buf[0..count])))
            }
        }
    }

    /// Flush the io
    pub fn sync(&mut self) -> bool {
        unsafe { sys_fsync(self.fd) == 0 }
    }

    pub fn set_len(&mut self, size: usize) -> bool {
        unsafe { sys_ftruncate(self.fd, size) == 0 }
    }
}

impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        unsafe {
            let count = sys_read(self.fd, buf.as_mut_ptr(), buf.len());
            if count == usize::MAX {
                None
            } else {
                Some(count)
            }
        }
    }
}

impl Write for File {
    fn write(&mut self, buf: &[u8]) -> Option<usize> {
        unsafe {
            let count = sys_write(self.fd, buf.as_ptr(), buf.len());
            if count == usize::MAX {
                None
            } else {
                Some(count)
            }
        }
    }
}

impl Seek for File {
    /// Seek a given position
    fn seek(&mut self, pos: SeekFrom) -> Option<usize> {
        let (whence, offset) = match pos {
            SeekFrom::Start(offset) => (SEEK_SET, offset as isize),
            SeekFrom::Current(offset) => (SEEK_CUR, offset),
            SeekFrom::End(offset) => (SEEK_END, offset),
        };

        let position = unsafe { sys_lseek(self.fd, offset, whence) };
        if position == usize::MAX {
            None
        } else {
            Some(position)
        }
    }
}

impl Drop for File {
    fn drop(&mut self) {
        unsafe {
            sys_close(self.fd);
        }
    }
}

pub struct DirEntry {
    path: String,
}

impl DirEntry {
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Create a new directory, using a path
    /// The default mode of the directory is 744
    pub fn create(path: &str) -> Option<DirEntry> {
        unsafe {
            let dir = sys_mkdir((path.to_string() + "\0").as_ptr(), 744);
            if dir == usize::MAX {
                None
            } else {
                Some(DirEntry { path: path.to_string() })
            }
        }
    }

}

pub struct ReadDir {
    file: File,
}

impl Iterator for ReadDir {
    type Item = DirEntry;
    fn next(&mut self) -> Option<DirEntry> {
        let mut path = String::new();
        let mut buf: [u8; 1] = [0; 1];
        loop {
            match self.file.read(&mut buf) {
                Some(0) => break,
                Some(count) => {
                    if buf[0] == 10 {
                        break;
                    } else {
                        path.push_str(unsafe { str::from_utf8_unchecked(&buf[..count]) });
                    }
                }
                None => break,
            }
        }
        if path.is_empty() {
            None
        } else {
            Some(DirEntry { path: path })
        }
    }
}

pub fn read_dir(path: &str) -> Option<ReadDir> {
    let file_option = if path.is_empty() || path.ends_with('/') {
        File::open(path)
    } else {
        File::open(&(path.to_string() + "/"))
    };

    if let Some(file) = file_option {
        Some(ReadDir { file: file })
    } else {
        None
    }
}

pub fn change_cwd(path: &str) -> bool {
    let file_option = if path.is_empty() || path.ends_with('/') {
        File::open(path)
    } else {
        File::open(&(path.to_string() + "/"))
    };

    if let Some(file) = file_option {
        if let Some(file_path) = file.path() {
            if unsafe { sys_chdir((file_path + "\0").as_ptr()) } == 0 {
                return true;
            }
        }
    }

    false
}
