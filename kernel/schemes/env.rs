use fs::{KScheme, Resource, Url};
use fs::resource::ResourceSeek;
use collections::string::String;
use alloc::boxed::Box;
use system::error::{Error, Result, EINVAL};
use core::cmp::min;

pub struct EnvScheme;

impl KScheme for EnvScheme {
    fn scheme(&self) -> &str {
        "env"
    }

    fn open(&mut self, url: Url, _: usize) -> Result<Box<Resource>> {
        let name = url.reference();
        if name.contains('=') { return Err(Error::new(EINVAL)) }
        Ok(box EnvResource {
            name: String::from(name),
            pos: 0
        })
    }
}

pub struct EnvResource {
    name: String,
    pos: usize
}

impl Resource for EnvResource {
    fn dup(&self) -> Result<Box<Resource>> {
        Ok(box EnvResource { name: self.name.clone(), pos: 0 })
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let contexts = ::env().contexts.lock();
        let current = try!(contexts.current());
        let value = try!(current.get_env_var(&self.name));
        let mut i = 0;
        while i < buf.len() && self.pos < value.bytes().count() {
            match value.bytes().nth(self.pos) {
                Some(c) => buf[i] = c as u8,
                None => ()
            }
            i += 1;
            self.pos += 1;
        }
        Ok(i)
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let mut contexts = ::env().contexts.lock();
        let current = try!(contexts.current_mut());
        let value = String::from_utf8_lossy(buf).into_owned();
        if value.contains('�') {
            return Err(Error::new(EINVAL));
        }
        try!(current.set_env_var(&self.name, &value));
        Ok(min(value.as_bytes().len(), buf.len()))
    }

    fn seek(&mut self, pos: ResourceSeek) -> Result<usize> {
        match pos {
            ResourceSeek::Start(offset) => self.pos = offset,
            ResourceSeek::Current(offset) => self.pos = (self.pos as isize + offset) as usize,
            ResourceSeek::End(offset) => {
                let contexts = ::env().contexts.lock();
                let current = try!(contexts.current());
                let value = try!(current.get_env_var(&self.name));
                self.pos = (value.bytes().count() as isize + offset) as usize;
            }
        }
        Ok(self.pos)
    }
}
