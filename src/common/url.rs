use core::clone::Clone;

use common::debug::*;
use common::string::*;
use common::vector::*;

pub struct URL {
    pub scheme: String,
    pub user: String,
    pub password: String,
    pub host: String,
    pub port: String,
    pub path: Vector<String>
}

impl URL {
    pub fn new() -> URL {
        URL {
            scheme: String::new(),
            user: String::new(),
            password: String::new(),
            host: String::new(),
            port: String::new(),
            path: Vector::new()
        }
    }

    pub fn from_string(url_string: String) -> URL {
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
                },
                _ => url.path.push(part)
            }
            part_i += 1;
        }

        return url;
    }

    pub fn to_string(&self) -> String{
        let mut ret = self.scheme.clone() + "://";

        if self.user.len() > 0 {
            ret = ret + self.user.clone();
            if self.password.len() > 0 {
                ret = ret + ":" + self.password.clone();
            }
            ret = ret + "@";
        }

        if self.host.len() > 0 {
            ret = ret + self.host.clone();
            if self.port.len() > 0 {
                ret = ret + ":" + self.port.clone();
            }
        }

        for element in self.path.as_slice() {
            ret = ret + "/" + element.clone();
        }

        return ret;
    }

    pub fn d(&self){
        self.to_string().d();
        dl();
    }
}
