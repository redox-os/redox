use syscall;
use syscall::data::{Stat, StatVfs};
use syscall::error::{Error, EBADF, EINVAL, EPERM, Result};
use syscall::scheme::Scheme;

use std::str;
use std::path::PathBuf;

pub struct ChrootScheme {
    root: PathBuf
}

impl ChrootScheme {
    pub fn new(root: PathBuf) -> ChrootScheme {
        ChrootScheme {
            root: root
        }
    }

    fn translate(&self, path: &[u8]) -> Result<String> {
        let path = str::from_utf8(path).or(Err(Error::new(EINVAL)))?;
        let mut translated = self.root.clone();
        translated.push(path.trim_left_matches('/'));
        if translated.starts_with(&self.root) {
            translated.into_os_string().into_string().or(Err(Error::new(EINVAL)))
        } else {
            println!("escaped chroot");
            Err(Error::new(EPERM))
        }
    }
}

impl Scheme for ChrootScheme {
    fn open(&self, path: &[u8], flags: usize, uid: u32, gid: u32) -> Result<usize> {
        if uid != 0 {
            syscall::setreuid(0, uid as usize)?;
        }
        if gid != 0 {
            syscall::setregid(0, gid as usize)?;
        }
        let res = syscall::open(&self.translate(path)?, flags);
        if uid != 0 {
            syscall::setreuid(0, 0).unwrap();
        }
        if gid != 0 {
            syscall::setregid(0, 0).unwrap();
        }
        res
    }

    fn chmod(&self, path: &[u8], mode: u16, uid: u32, gid: u32) -> Result<usize> {
        if uid != 0 {
            syscall::setreuid(0, uid as usize)?;
        }
        if gid != 0 {
            syscall::setregid(0, gid as usize)?;
        }
        let res = syscall::chmod(&self.translate(path)?, mode as usize);
        if uid != 0 {
            syscall::setreuid(0, 0).unwrap();
        }
        if gid != 0 {
            syscall::setregid(0, 0).unwrap();
        }
        res
    }

    fn rmdir(&self, path: &[u8], uid: u32, gid: u32) -> Result<usize> {
        if uid != 0 {
            syscall::setreuid(0, uid as usize)?;
        }
        if gid != 0 {
            syscall::setregid(0, gid as usize)?;
        }
        let res = syscall::rmdir(&self.translate(path)?);
        if uid != 0 {
            syscall::setreuid(0, 0).unwrap();
        }
        if gid != 0 {
            syscall::setregid(0, 0).unwrap();
        }
        res
    }

    fn unlink(&self, path: &[u8], uid: u32, gid: u32) -> Result<usize> {
        if uid != 0 {
            syscall::setreuid(0, uid as usize)?;
        }
        if gid != 0 {
            syscall::setregid(0, gid as usize)?;
        }
        let res = syscall::unlink(&self.translate(path)?);
        if uid != 0 {
            syscall::setreuid(0, 0).unwrap();
        }
        if gid != 0 {
            syscall::setregid(0, 0).unwrap();
        }
        res
    }

    /* Resource operations */
    fn dup(&self, old_id: usize, buf: &[u8]) -> Result<usize> {
        syscall::dup(old_id, buf)
    }

    fn read(&self, id: usize, buf: &mut [u8]) -> Result<usize> {
        syscall::read(id, buf)
    }

    fn write(&self, id: usize, buf: &[u8]) -> Result<usize> {
        syscall::write(id, buf)
    }

    fn seek(&self, id: usize, pos: usize, whence: usize) -> Result<usize> {
        syscall::lseek(id, pos as isize, whence)
    }

    fn fcntl(&self, id: usize, cmd: usize, arg: usize) -> Result<usize> {
        syscall::fcntl(id, cmd, arg)
    }

    fn fevent(&self, _id: usize, _flags: usize) -> Result<usize> {
        //TODO
        Err(Error::new(EBADF))
    }

    fn fmap(&self, _id: usize, _offset: usize, _size: usize) -> Result<usize> {
        //TODO
        Err(Error::new(EBADF))
    }

    fn fpath(&self, id: usize, buf: &mut [u8]) -> Result<usize> {
        let count = syscall::fpath(id, buf)?;

        let translated = {
            let path = str::from_utf8(&buf[.. count]).or(Err(Error::new(EINVAL)))?;
            let translated = path.to_string().replace(self.root.to_str().ok_or(Error::new(EINVAL))?, "");
            format!("file:{}", translated.trim_left_matches('/'))
        };

        let path = translated.as_bytes();

        let mut i = 0;
        while i < buf.len() && i < path.len() {
            buf[i] = path[i];
            i += 1;
        }

        Ok(i)
    }

    fn fstat(&self, id: usize, stat: &mut Stat) -> Result<usize> {
        syscall::fstat(id, stat)
    }

    fn fstatvfs(&self, id: usize, stat: &mut StatVfs) -> Result<usize> {
        syscall::fstatvfs(id, stat)
    }

    fn fsync(&self, id: usize) -> Result<usize> {
        syscall::fsync(id)
    }

    fn ftruncate(&self, id: usize, len: usize) -> Result<usize> {
        syscall::ftruncate(id, len)
    }

    fn close(&self, id: usize) -> Result<usize> {
        syscall::close(id)
    }
}
