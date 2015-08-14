use alloc::boxed::*;

use core::cmp::min;
use core::cmp::max;

use common::vec::*;

pub enum ResourceSeek {
    Start(usize),
    End(isize),
    Current(isize),
}

pub enum ResourceType {
    None,
    Array,
    Dir,
    File
}

#[allow(unused_variables)]
pub trait Resource {
    fn stat(&self) -> ResourceType {
        return ResourceType::None;
    }

    fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        return Option::None;
    }

    fn read_async(&mut self, buf: &mut [u8], callback: Box<FnBox(Option<usize>)>){
        callback.call_box((self.read(buf),));
    }

    fn write(&mut self, buf: &[u8]) -> Option<usize> {
        return Option::None;
    }

    fn write_async(&mut self, buf: &[u8], callback: Box<FnBox(Option<usize>)>){
        callback.call_box((self.write(buf),));
    }

    fn seek(&mut self, pos: ResourceSeek) -> Option<usize> {
        return Option::None;
    }
}

pub struct NoneResource;

impl Resource for NoneResource {}

pub struct VecResource {
    vec: Vec<u8>,
    seek: usize
}

impl VecResource {
    pub fn new(vec: Vec<u8>) -> VecResource {
        return VecResource {
            vec: vec,
            seek: 0
        };
    }
}

impl Resource for VecResource {
    fn stat(&self) -> ResourceType {
        return ResourceType::File;
    }

    fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        let mut i = 0;
        while i < buf.len() && self.seek < self.vec.len() {
            match self.vec.get(self.seek) {
                Option::Some(b) => buf[i] = *b,
                Option::None => ()
            }
            self.seek += 1;
            i += 1;
        }
        return Option::Some(i);
    }

    fn write(&mut self, buf: &[u8]) -> Option<usize> {
        let mut i = 0;
        while i < buf.len() && self.seek < self.vec.len() {
            self.vec.set(self.seek, buf[i]);
            self.seek += 1;
            i += 1;
        }
        return Option::Some(i);
    }

    fn seek(&mut self, pos: ResourceSeek) -> Option<usize> {
        match pos {
            ResourceSeek::Start(offset) => self.seek = min(self.seek, offset),
            ResourceSeek::End(offset) => self.seek = max(0, min(self.seek as isize, self.vec.len() as isize + offset)) as usize,
            ResourceSeek::Current(offset) => self.seek = max(0, min(self.seek as isize, self.seek as isize + offset)) as usize
        }
        return Option::Some(self.seek);
    }
}
