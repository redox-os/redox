use core::clone::Clone;
use core::mem;

use string::{String, ToString};
use vec::Vec;

// URL Parsing:
// Split by /
// scheme://user:password@host:port/path/path/path?query#fragment
// First part is scheme, second is empty, third is user, password, host, and port, later parts are path, last part is path, query, and fragment
// Split third part by @, the last part is the host and port, if there is a first part it is the user and password
// Split these parts each by :, first part splits into user and password, the second part is split into domain and port
// Split the last part by ?, the first part is a path element, the last part is the query and fragment
// Split the last part by #, the first is the query, the second is the fragment
// Split the query by &

/// An URL, see wiki
pub struct Url {
    pub string: String,
}

impl Url {
    /// Create a new empty URL
    pub fn new() -> Self {
        Url { string: String::new() }
    }

    /// Create an URL from a string literal
    pub fn from_str(url_str: &str) -> Self {
        return Url::from_string(url_str.to_string());
    }

    /// Create an URL from `String`
    pub fn from_string(url_string: String) -> Self {
        Url { string: url_string }
    }

    /// Convert to string
    pub fn to_string(&self) -> String {
        return self.string.clone();
    }

    /// Get the length of this URL
    pub fn len(&self) -> usize {
        return self.string.len();
    }

    /// Return the scheme of this url
    pub fn scheme(&self) -> String {
        if let Some(part) = self.string.split('/').next() {
            if let Some(scheme_part) = part.split(':').next() {
                return scheme_part.to_string();
            }
        }
        return String::new();
    }

    /// Get the owner's username (the conventional @)
    pub fn username(&self) -> String {
        let mut username = String::new();
        let mut host = String::new();

        let mut part_i = 0;
        for part in self.string.split('/') {
            match part_i {
                0 => (),
                1 => (),
                2 => {
                    let mut host_part_i = 0;
                    for host_part in part.split('@') {
                        let mut host_subpart_i = 0;
                        for host_subpart in host_part.split(':') {
                            match host_part_i {
                                0 => {
                                    match host_subpart_i {
                                        0 => username = host_subpart.to_string(),
                                        _ => (),
                                    }
                                }
                                1 => {
                                    match host_subpart_i {
                                        0 => host = host_subpart.to_string(),
                                        _ => (),
                                    }
                                }
                                _ => (),
                            }
                            host_subpart_i += 1;
                        }
                        host_part_i += 1;
                    }
                    if host_part_i == 1 {
                        mem::swap(&mut host, &mut username);
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
        for part in self.string.split('/') {
            match part_i {
                0 => (),
                1 => (),
                2 => {
                    let mut host_part_i = 0;
                    for host_part in part.split('@') {
                        let mut host_subpart_i = 0;
                        for host_subpart in host_part.split(':') {
                            match host_part_i {
                                0 => {
                                    match host_subpart_i {
                                        1 => password = host_subpart.to_string(),
                                        _ => (),
                                    }
                                }
                                1 => {
                                    match host_subpart_i {
                                        1 => port = host_subpart.to_string(),
                                        _ => (),
                                    }
                                }
                                _ => (),
                            }
                            host_subpart_i += 1;
                        }
                        host_part_i += 1;
                    }
                    if host_part_i == 1 {
                        mem::swap(&mut port, &mut password);
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
        for part in self.string.split('/') {
            match part_i {
                0 => (),
                1 => (),
                2 => {
                    let mut host_part_i = 0;
                    for host_part in part.split('@') {
                        let mut host_subpart_i = 0;
                        for host_subpart in host_part.split(':') {
                            match host_part_i {
                                0 => {
                                    match host_subpart_i {
                                        0 => username = host_subpart.to_string(),
                                        _ => (),
                                    }
                                }
                                1 => {
                                    match host_subpart_i {
                                        0 => host = host_subpart.to_string(),
                                        _ => (),
                                    }
                                }
                                _ => (),
                            }
                            host_subpart_i += 1;
                        }
                        host_part_i += 1;
                    }
                    if host_part_i == 1 {
                        mem::swap(&mut host, &mut username);
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
        for part in self.string.split('/') {
            match part_i {
                0 => (),
                1 => (),
                2 => {
                    let mut host_part_i = 0;
                    for host_part in part.split('@') {
                        let mut host_subpart_i = 0;
                        for host_subpart in host_part.split(':') {
                            match host_part_i {
                                0 => {
                                    match host_subpart_i {
                                        1 => password = host_subpart.to_string(),
                                        _ => (),
                                    }
                                }
                                1 => {
                                    match host_subpart_i {
                                        1 => port = host_subpart.to_string(),
                                        _ => (),
                                    }
                                }
                                _ => (),
                            }
                            host_subpart_i += 1;
                        }
                        host_part_i += 1;
                    }
                    if host_part_i == 1 {
                        mem::swap(&mut port, &mut password);
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
        for part in self.string.split('/') {
            match part_i {
                0 => (),
                1 => (),
                2 => (),
                3 => path = part.to_string(),
                _ => path = path + "/" + part,
            }
            part_i += 1;
        }

        // Hack for folders
        if part_i > 3 && self.string.ends_with('/') {
            path = path + "/";
        }

        return path;
    }

    /// Return the parts of the path
    pub fn path_parts(&self) -> Vec<String> {
        let mut path_parts: Vec<String> = Vec::new();

        let mut part_i = 0;
        for part in self.string.split('/') {
            match part_i {
                0 => (),
                1 => (),
                2 => (),
                _ => path_parts.push(part.to_string()),
            }
            part_i += 1;
        }

        return path_parts;
    }
}

impl Clone for Url {
    fn clone(&self) -> Self {
        Url { string: self.string.clone() }
    }
}
