use alloc::boxed::Box;
use arch::context::EnvVar;
use collections::string::String;
use core::cmp::min;
use fs::resource::ResourceSeek;
use fs::{KScheme, Resource, Url};
use system::error::{EINVAL, Error, Result};

pub struct EnvScheme;

impl KScheme for EnvScheme {
    fn scheme(&self) -> &str {
        "env"
    }

    fn open(&mut self, url: Url, _: usize) -> Result<Box<Resource>> {
        let name = url.reference();
        if name.contains('=') { return Err(Error::new(EINVAL)) }
        if name == "" || name == "/" {
            Ok(box EnvListResource {
                pos: 0
            })
        } else {
            Ok(box EnvVariableResource {
                name: String::from(name),
                pos: 0
            })
        }
    }

    fn unlink(&mut self, url: Url) -> Result<()> {
        let name = url.reference();
        let contexts = ::env().contexts.lock();
        let current = try!(contexts.current());
        current.remove_env_var(name)
    }
}

pub struct EnvListResource {
    pos: usize
}

impl EnvListResource {
    fn get_list_str(&self) -> Result<String> {
        let contexts = ::env().contexts.lock();
        let current = contexts.current()?;
        let values = current.list_env_vars();
        let mut string = String::new();
        for &EnvVar(ref name, ref value) in values.iter() {
            string = string + name + "=" + value + "\n";
        }
        string.pop();
        Ok(string)
    }
}

impl Resource for EnvListResource {
    fn dup(&self) -> Result<Box<Resource>> {
        Ok(box EnvListResource { pos: 0 })
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut i = 0;
        let string = try!(self.get_list_str());
        while i < buf.len() && self.pos < string.bytes().count() {
            match string.bytes().nth(self.pos) {
                Some(c) => buf[i] = c,
                None => ()
            }
            i += 1;
            self.pos += 1;
        }
        Ok(i)
    }

    fn seek(&mut self, pos: ResourceSeek) -> Result<usize> {
        match pos {
            ResourceSeek::Start(offset) => self.pos = offset,
            ResourceSeek::Current(offset) => self.pos = (self.pos as isize + offset) as usize,
            ResourceSeek::End(offset) => {
                let string = try!(self.get_list_str());
                self.pos = (string.bytes().count() as isize + offset) as usize;
            }
        }
        Ok(self.pos)
    }
}

pub struct EnvVariableResource {
    name: String,
    pos: usize
}

impl Resource for EnvVariableResource {
    fn dup(&self) -> Result<Box<Resource>> {
        Ok(box EnvVariableResource { name: self.name.clone(), pos: 0 })
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let contexts = ::env().contexts.lock();
        let current = try!(contexts.current());
        let value = try!(current.get_env_var(&self.name));
        let mut i = 0;
        while i < buf.len() && self.pos < value.bytes().count() {
            match value.bytes().nth(self.pos) {
                Some(c) => buf[i] = c,
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
