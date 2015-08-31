use alloc::boxed::*;

use core::clone::Clone;
use core::cmp::min;
use core::cmp::max;
use core::ptr;

use common::debug::*;
use common::memory::*;
use common::string::*;
use common::vec::*;

use syscall::call::sys_open;

pub enum ResourceSeek {
    Start(usize),
    End(isize),
    Current(isize),
}

#[derive(Copy, Clone)]
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

    fn read_to_end(&mut self, vec: &mut Vec<u8>) -> Option<usize> {
        return Option::None;
    }

    fn write(&mut self, buf: &[u8]) -> Option<usize> {
        return Option::None;
    }

    fn write_all(&mut self, vec: &Vec<u8>) -> Option<usize> {
        return Option::None;
    }

    fn seek(&mut self, pos: ResourceSeek) -> Option<usize> {
        return Option::None;
    }

    fn flush(&mut self) -> bool {
        return true;
    }
}

pub struct URL {
    pub scheme: String,
    /*
    pub user: String,
    pub password: String,
    pub host: String,
    pub port: String,
    */
    pub path: String
}

impl URL {
    pub fn new() -> URL {
        URL {
            scheme: String::new(),
            /*
            user: String::new(),
            password: String::new(),
            host: String::new(),
            port: String::new(),
            */
            path: String::new()
        }
    }

    pub fn from_string(url_string: &String) -> URL {
        let mut url = URL::new();

        //Split by /
        //First part is scheme, second is empty, third is user, password, host, and port, later parts are path, last part is path, query, and fragment
            //Split third part by @, the last part is the host and port, if there is a first part it is the user and password
                //Split these parts each by :, first part splits into user and password, the second part is split into domain and port
            //Split the last part by ?, the first part is a path element, the last part is the query and fragment
                //Split the last part by #, the first is the query, the second is the fragment
                    //Split the query by &

        let mut part_i = 0;
        for part in url_string.split("/".to_string()) {
            match part_i {
                0 => {
                    let mut scheme_part_i = 0;
                    for scheme_part in part.split(":".to_string()) {
                        match scheme_part_i {
                            0 => url.scheme = scheme_part,
                            _ => ()
                        }
                        scheme_part_i += 1;
                    }
                }
                1 => (),
                2 => {
                    /*
                    let mut host_part_i = 0;
                    for host_part in part.split("@".to_string()){
                        let mut host_subpart_i = 0;
                        for host_subpart in host_part.split(":".to_string()) {
                            match host_part_i {
                                0 => match host_subpart_i {
                                    0 => url.user = host_subpart,
                                    1 => url.password = host_subpart,
                                    _ => ()
                                },
                                1 => match host_subpart_i {
                                    0 => url.host = host_subpart,
                                    1 => url.port = host_subpart,
                                    _ => ()
                                },
                                _ => ()
                            }
                            host_subpart_i += 1;
                        }
                        host_part_i += 1;
                    }
                    if host_part_i == 1 {
                        url.host = url.user;
                        url.user = String::new();
                        url.port = url.password;
                        url.password = String::new();
                    }
                    */
                },
                3 => url.path = part,
                _ => url.path = url.path + "/" + part
            }
            part_i += 1;
        }

        return url;
    }

    pub fn open(&self) -> Box<Resource> {
        unsafe{
            let resource_ptr: *mut Box<Resource> = alloc_type();

            sys_open(self, resource_ptr);

            let ret = ptr::read(resource_ptr);

            unalloc(resource_ptr as usize);

            return ret;
        }
    }

    pub fn to_string(&self) -> String {
        let mut ret = self.scheme.clone() + "://";

        /*
        if self.user.len() > 0 {
            ret = ret + &self.user;
            if self.password.len() > 0 {
                ret = ret + ":" + &self.password;
            }
            ret = ret + "@";
        }

        if self.host.len() > 0 {
            ret = ret + &self.host;
            if self.port.len() > 0 {
                ret = ret + ":" + &self.port;
            }
        }
        */

        if self.path.len() > 0 {
            ret = ret + "/" + &self.path;
        }

        return ret;
    }

    pub fn d(&self){
        self.to_string().d();
        dl();
    }
}

impl Clone for URL {
    fn clone(&self) -> URL{
        URL {
            scheme: self.scheme.clone(),
            /*
            user: self.user.clone(),
            password: self.password.clone(),
            host: self.host.clone(),
            port: self.port.clone(),
            */
            path: self.path.clone()
        }
    }
}

pub struct NoneResource;

impl Resource for NoneResource {}

pub struct VecResource {
    resource_type: ResourceType,
    vec: Vec<u8>,
    seek: usize
}

impl VecResource {
    pub fn new(resource_type: ResourceType, vec: Vec<u8>) -> VecResource {
        return VecResource {
            resource_type: resource_type,
            vec: vec,
            seek: 0
        };
    }

    pub fn inner(&self) -> &Vec<u8> {
        return &self.vec;
    }
}

impl Resource for VecResource {
    fn stat(&self) -> ResourceType {
        return self.resource_type;
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

    fn read_to_end(&mut self, vec: &mut Vec<u8>) -> Option<usize> {
        vec.push_all(&self.vec);
        return Option::Some(self.vec.len());
    }

    fn write(&mut self, buf: &[u8]) -> Option<usize> {
        let mut i = 0;
        while i < buf.len() && self.seek < self.vec.len() {
            self.vec.set(self.seek, buf[i]);
            self.seek += 1;
            i += 1;
        }
        while i < buf.len() {
            self.vec.push(buf[i]);
            self.seek += 1;
            i += 1;
        }
        return Option::Some(i);
    }

    fn write_all(&mut self, vec: &Vec<u8>) -> Option<usize> {
        let mut i = 0;
        while i < vec.len() && self.seek < self.vec.len() {
            match vec.get(i) {
                Option::Some(b) => {
                    self.vec.set(self.seek, *b);
                    self.seek += 1;
                    i += 1;
                },
                Option::None => break
            }
        }
        while i < vec.len() {
            match vec.get(i) {
                Option::Some(b) => {
                    self.vec.push(*b);
                    self.seek += 1;
                    i += 1;
                },
                Option::None => break
            }
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
