use alloc::boxed::*;

use core::clone::Clone;
use core::cmp::min;
use core::cmp::max;
use core::mem::swap;
use core::ptr;

use common::debug::*;
use common::memory::*;
use common::string::*;
use common::vec::*;

use syscall::call::*;

pub enum ResourceSeek {
    Start(usize),
    Current(isize),
    End(isize)
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
    fn url(&self) -> URL;
    fn stat(&self) -> ResourceType;
    fn read(&mut self, buf: &mut [u8]) -> Option<usize>;
    fn read_to_end(&mut self, vec: &mut Vec<u8>) -> Option<usize>;
    fn write(&mut self, buf: &[u8]) -> Option<usize>;
    fn seek(&mut self, pos: ResourceSeek) -> Option<usize>;
    fn flush(&mut self) -> bool;
}

//URL Parsing:
//Split by /
//First part is scheme, second is empty, third is user, password, host, and port, later parts are path, last part is path, query, and fragment
    //Split third part by @, the last part is the host and port, if there is a first part it is the user and password
        //Split these parts each by :, first part splits into user and password, the second part is split into domain and port
    //Split the last part by ?, the first part is a path element, the last part is the query and fragment
        //Split the last part by #, the first is the query, the second is the fragment
            //Split the query by &
pub struct URL {
    pub string: String
}

impl URL {
    pub fn new() -> URL {
        URL {
            string: String::new()
        }
    }

    pub fn from_str(url_str: &'static str) -> URL {
        return URL::from_string(&url_str.to_string());
    }

    pub fn from_string(url_string: &String) -> URL {
        URL {
            string: url_string.clone()
        }
    }

    pub fn to_string(&self) -> String {
        return self.string.clone();
    }

    pub fn len(&self) -> usize {
        return self.string.len();
    }

    pub fn d(&self){
        self.string.d();
    }

    pub fn open(&self) -> Box<Resource> {
        unsafe{
            return (*::session_ptr).open(&self);
        }
    }

    pub fn scheme(&self) -> String {
        let mut part_i = 0;
        for part in self.string.split("/".to_string()) {
            match part_i {
                0 => {
                        let mut scheme_part_i = 0;
                        for scheme_part in part.split(":".to_string()) {
                            match scheme_part_i {
                                0 => return scheme_part,
                                _ => break
                            }
                            scheme_part_i += 1;
                        }
                },
                _ => break
            }
            part_i += 1;
        }

        return String::new();
    }

    pub fn username(&self) -> String {
        let mut username = String::new();
        let mut host = String::new();

        let mut part_i = 0;
        for part in self.string.split("/".to_string()) {
            match part_i {
                0 => (),
                1 => (),
                2 => {
                    let mut host_part_i = 0;
                    for host_part in part.split("@".to_string()){
                        let mut host_subpart_i = 0;
                        for host_subpart in host_part.split(":".to_string()) {
                            match host_part_i {
                                0 => match host_subpart_i {
                                    0 => username = host_subpart,
                                    _ => ()
                                },
                                1 => match host_subpart_i {
                                    0 => host = host_subpart,
                                    _ => ()
                                },
                                _ => ()
                            }
                            host_subpart_i += 1;
                        }
                        host_part_i += 1;
                    }
                    if host_part_i == 1 {
                        swap(&mut host, &mut username);
                    }
                },
                _ => break
            }
            part_i += 1;
        }

        return username;
    }

    pub fn password(&self) -> String {
        let mut password = String::new();
        let mut port = String::new();

        let mut part_i = 0;
        for part in self.string.split("/".to_string()) {
            match part_i {
                0 => (),
                1 => (),
                2 => {
                    let mut host_part_i = 0;
                    for host_part in part.split("@".to_string()){
                        let mut host_subpart_i = 0;
                        for host_subpart in host_part.split(":".to_string()) {
                            match host_part_i {
                                0 => match host_subpart_i {
                                    1 => password = host_subpart,
                                    _ => ()
                                },
                                1 => match host_subpart_i {
                                    1 => port = host_subpart,
                                    _ => ()
                                },
                                _ => ()
                            }
                            host_subpart_i += 1;
                        }
                        host_part_i += 1;
                    }
                    if host_part_i == 1 {
                        swap(&mut port, &mut password);
                    }
                },
                _ => break
            }
            part_i += 1;
        }

        return password;
    }

    pub fn host(&self) -> String {
        let mut username = String::new();
        let mut host = String::new();

        let mut part_i = 0;
        for part in self.string.split("/".to_string()) {
            match part_i {
                0 => (),
                1 => (),
                2 => {
                    let mut host_part_i = 0;
                    for host_part in part.split("@".to_string()){
                        let mut host_subpart_i = 0;
                        for host_subpart in host_part.split(":".to_string()) {
                            match host_part_i {
                                0 => match host_subpart_i {
                                    0 => username = host_subpart,
                                    _ => ()
                                },
                                1 => match host_subpart_i {
                                    0 => host = host_subpart,
                                    _ => ()
                                },
                                _ => ()
                            }
                            host_subpart_i += 1;
                        }
                        host_part_i += 1;
                    }
                    if host_part_i == 1 {
                        swap(&mut host, &mut username);
                    }
                },
                _ => break
            }
            part_i += 1;
        }

        return host;
    }

    pub fn port(&self) -> String {
        let mut password = String::new();
        let mut port = String::new();

        let mut part_i = 0;
        for part in self.string.split("/".to_string()) {
            match part_i {
                0 => (),
                1 => (),
                2 => {
                    let mut host_part_i = 0;
                    for host_part in part.split("@".to_string()){
                        let mut host_subpart_i = 0;
                        for host_subpart in host_part.split(":".to_string()) {
                            match host_part_i {
                                0 => match host_subpart_i {
                                    1 => password = host_subpart,
                                    _ => ()
                                },
                                1 => match host_subpart_i {
                                    1 => port = host_subpart,
                                    _ => ()
                                },
                                _ => ()
                            }
                            host_subpart_i += 1;
                        }
                        host_part_i += 1;
                    }
                    if host_part_i == 1 {
                        swap(&mut port, &mut password);
                    }
                },
                _ => break
            }
            part_i += 1;
        }

        return port;
    }

    pub fn path(&self) -> String {
        let mut path = String::new();

        let mut part_i = 0;
        for part in self.string.split("/".to_string()) {
            match part_i {
                0 => (),
                1 => (),
                2 => (),
                3 => path = part,
                _ => path = path + "/" + part
            }
            part_i += 1;
        }

        return path;
    }

    pub fn path_parts(&self) -> Vec<String> {
        let mut path_parts: Vec<String> = Vec::new();

        let mut part_i = 0;
        for part in self.string.split("/".to_string()) {
            match part_i {
                0 => (),
                1 => (),
                2 => (),
                _ => path_parts.push(part)
            }
            part_i += 1;
        }

        return path_parts;
    }
}

impl Clone for URL {
    fn clone(&self) -> URL{
        URL {
            string: self.string.clone()
        }
    }
}

pub struct NoneResource;

impl Resource for NoneResource {
    fn url(&self) -> URL {
        return URL::from_str("none://");
    }

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

    fn seek(&mut self, pos: ResourceSeek) -> Option<usize> {
        return Option::None;
    }

    fn flush(&mut self) -> bool {
        return false;
    }
}

pub struct VecResource {
    url: URL,
    resource_type: ResourceType,
    vec: Vec<u8>,
    seek: usize
}

impl VecResource {
    pub fn new(url: URL, resource_type: ResourceType, vec: Vec<u8>) -> VecResource {
        return VecResource {
            url: url,
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
    fn url(&self) -> URL {
        return self.url.clone();
    }

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

    fn seek(&mut self, pos: ResourceSeek) -> Option<usize> {
        match pos {
            ResourceSeek::Start(offset) => self.seek = min(self.seek, offset),
            ResourceSeek::End(offset) => self.seek = max(0, min(self.seek as isize, self.vec.len() as isize + offset)) as usize,
            ResourceSeek::Current(offset) => self.seek = max(0, min(self.seek as isize, self.seek as isize + offset)) as usize
        }
        return Option::Some(self.seek);
    }

    fn flush(&mut self) -> bool {
        return true;
    }
}
