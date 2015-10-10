use alloc::boxed::*;

use core::clone::Clone;
use core::cmp::{max, min};
use core::mem::swap;

use common::string::*;
use common::vec::*;

/// Resource seek
pub enum ResourceSeek {
    /// Start point
    Start(usize),
    /// Current point
    Current(isize),
    /// End point
    End(isize),
}

/// A resource type
#[derive(Copy, Clone)]
pub enum ResourceType {
    None,
    Array,
    Dir,
    File,
}

/// A system resource
#[allow(unused_variables)]
pub trait Resource {
    //Required functions
    /// Return the url of this resource
    fn url(&self) -> URL;
    /// Return the type of this resource
    fn stat(&self) -> ResourceType;
    // TODO: Make use of Write and Read trait
    /// Read data to buffer
    fn read(&mut self, buf: &mut [u8]) -> Option<usize>;
    /// Write to resource
    fn write(&mut self, buf: &[u8]) -> Option<usize>;
    /// Seek
    fn seek(&mut self, pos: ResourceSeek) -> Option<usize>;
    /// Sync the resource
    fn sync(&mut self) -> bool;

    //Helper functions
    fn read_to_end(&mut self, vec: &mut Vec<u8>) -> Option<usize> {
        let mut read = 0;
        loop {
            let mut bytes = [0; 1024];
            match self.read(&mut bytes) {
                Option::Some(0) => return Option::Some(read),
                Option::None => return Option::None,
                Option::Some(count) => {
                    for i in 0..count {
                        vec.push(bytes[i]);
                    }
                    read += count;
                }
            }
        }
    }
}

//URL Parsing:
//Split by /
//scheme://user:password@host:port/path/path/path?query#fragment
//First part is scheme, second is empty, third is user, password, host, and port, later parts are path, last part is path, query, and fragment
    //Split third part by @, the last part is the host and port, if there is a first part it is the user and password
        //Split these parts each by :, first part splits into user and password, the second part is split into domain and port
    //Split the last part by ?, the first part is a path element, the last part is the query and fragment
        //Split the last part by #, the first is the query, the second is the fragment
            //Split the query by &

/// An URL, see wiki
pub struct URL {
    pub string: String,
}

impl URL {
    /// Create a new empty URL
    pub fn new() -> Self {
        URL { string: String::new() }
    }

    /// Create an URL from a string literal
    pub fn from_str(url_str: &'static str) -> Self {
        return URL::from_string(&url_str.to_string());
    }

    /// Create an URL from `String`
    pub fn from_string(url_string: &String) -> Self {
        URL { string: url_string.clone() }
    }

    /// Convert to string
    pub fn to_string(&self) -> String {
        return self.string.clone();
    }

    /// Get the length of this URL
    pub fn len(&self) -> usize {
        return self.string.len();
    }

    // FIXME: Strange naming.
    pub fn d(&self) {
        self.string.d();
    }

    /// Open this URL (returns a resource)
    pub fn open(&self) -> Box<Resource> {
        unsafe {
            return (*::session_ptr).open(&self);
        }
    }

    /// Return the scheme of this url
    pub fn scheme(&self) -> String {
        if let Some(part) = self.string.split("/".to_string()).next() {
            if let Some(scheme_part) = part.split(":".to_string()).next() {
                return scheme_part;
            }
        }
        return String::new();
    }

    /// Get the owner's username (the conventional @)
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
                    for host_part in part.split("@".to_string()) {
                        let mut host_subpart_i = 0;
                        for host_subpart in host_part.split(":".to_string()) {
                            match host_part_i {
                                0 => match host_subpart_i {
                                    0 => username = host_subpart,
                                    _ => (),
                                },
                                1 => match host_subpart_i {
                                    0 => host = host_subpart,
                                    _ => (),
                                },
                                _ => (),
                            }
                            host_subpart_i += 1;
                        }
                        host_part_i += 1;
                    }
                    if host_part_i == 1 {
                        swap(&mut host, &mut username);
                    }
                }
                _ => break,
            }
            part_i += 1;
        }

        username
    }

    /// Get the password from the url
    // TODO: Should probably be hashed?
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
                    for host_part in part.split("@".to_string()) {
                        let mut host_subpart_i = 0;
                        for host_subpart in host_part.split(":".to_string()) {
                            match host_part_i {
                                0 => match host_subpart_i {
                                    1 => password = host_subpart,
                                    _ => (),
                                },
                                1 => match host_subpart_i {
                                    1 => port = host_subpart,
                                    _ => (),
                                },
                                _ => (),
                            }
                            host_subpart_i += 1;
                        }
                        host_part_i += 1;
                    }
                    if host_part_i == 1 {
                        swap(&mut port, &mut password);
                    }
                }
                _ => break,
            }
            part_i += 1;
        }

        return password;
    }

    /// Get the host
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
                    for host_part in part.split("@".to_string()) {
                        let mut host_subpart_i = 0;
                        for host_subpart in host_part.split(":".to_string()) {
                            match host_part_i {
                                0 => match host_subpart_i {
                                    0 => username = host_subpart,
                                    _ => (),
                                },
                                1 => match host_subpart_i {
                                    0 => host = host_subpart,
                                    _ => (),
                                },
                                _ => (),
                            }
                            host_subpart_i += 1;
                        }
                        host_part_i += 1;
                    }
                    if host_part_i == 1 {
                        swap(&mut host, &mut username);
                    }
                }
                _ => break,
            }
            part_i += 1;
        }

        return host;
    }

    /// Get the post of the url
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
                    for host_part in part.split("@".to_string()) {
                        let mut host_subpart_i = 0;
                        for host_subpart in host_part.split(":".to_string()) {
                            match host_part_i {
                                0 => match host_subpart_i {
                                    1 => password = host_subpart,
                                    _ => (),
                                },
                                1 => match host_subpart_i {
                                    1 => port = host_subpart,
                                    _ => (),
                                },
                                _ => (),
                            }
                            host_subpart_i += 1;
                        }
                        host_part_i += 1;
                    }
                    if host_part_i == 1 {
                        swap(&mut port, &mut password);
                    }
                }
                _ => break,
            }
            part_i += 1;
        }

        return port;
    }

    /// Get the path of the url
    pub fn path(&self) -> String {
        let mut path = String::new();

        let mut part_i = 0;
        for part in self.string.split("/".to_string()) {
            match part_i {
                0 => (),
                1 => (),
                2 => (),
                3 => path = part,
                _ => path = path + "/" + part,
            }
            part_i += 1;
        }

        //Hack for folders
        if part_i > 3 && self.string.ends_with("/".to_string()) {
            path = path + "/";
        }

        return path;
    }

    /// Return the parts of the path
    pub fn path_parts(&self) -> Vec<String> {
        let mut path_parts: Vec<String> = Vec::new();

        let mut part_i = 0;
        for part in self.string.split("/".to_string()) {
            match part_i {
                0 => (),
                1 => (),
                2 => (),
                _ => path_parts.push(part),
            }
            part_i += 1;
        }

        return path_parts;
    }
}

impl Clone for URL {
    fn clone(&self) -> Self {
        URL { string: self.string.clone() }
    }
}

/// Empty resource
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

    fn write(&mut self, buf: &[u8]) -> Option<usize> {
        return Option::None;
    }

    fn seek(&mut self, pos: ResourceSeek) -> Option<usize> {
        return Option::None;
    }

    fn sync(&mut self) -> bool {
        return false;
    }
}

/// A vector resource
pub struct VecResource {
    url: URL,
    resource_type: ResourceType,
    vec: Vec<u8>,
    seek: usize,
}

impl VecResource {
    pub fn new(url: URL, resource_type: ResourceType, vec: Vec<u8>) -> Self {
        return VecResource {
            url: url,
            resource_type: resource_type,
            vec: vec,
            seek: 0,
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
                Option::None => (),
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
        while i < buf.len() {
            self.vec.push(buf[i]);
            self.seek += 1;
            i += 1;
        }
        return Option::Some(i);
    }

    fn seek(&mut self, pos: ResourceSeek) -> Option<usize> {
        match pos {
            ResourceSeek::Start(offset) => self.seek = min(self.vec.len(), offset),
            ResourceSeek::Current(offset) =>
                self.seek = max(0, min(self.seek as isize, self.seek as isize + offset)) as usize,
            ResourceSeek::End(offset) =>
                self.seek =
                    max(0, min(self.seek as isize, self.vec.len() as isize + offset)) as usize,
        }
        return Option::Some(self.seek);
    }

    fn sync(&mut self) -> bool {
        return true;
    }
}
