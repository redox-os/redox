use common::*;

mod rsa;

/// An installable package for Ox
pub struct Package {
    // TODO: Many of these uses String where other types can be used (when integreting with octavo)
    /// The host (where the package can be found)
    pub host: String,
    /// The name of this package
    pub name: String,
    /// The version of this package
    pub version: String,
    /// Description
    pub desc: String,
    //    /// The developer's public key
    //    pub dev: String,
    //    /// The developer's signature of this package
    //    pub dev_signature: String,
    //    /// Signatures of the developers public key from people who trusts the developers.
    //    // TODO: Move this to other file
    //    pub trust: Vec<String>,
    //    /// The signatures of this package
    //    pub signatures: Vec<String>,
    /// The files this package will create on the computer.
    pub files: Vec<String>,
}

pub fn get_package(host: String, name: String, version: String) -> Package {
    let con = File::open("tcp://".to_string() + host);

    con.write("GET /ox/".to_string() + version + "-".to_string() + name + " HTTP/1.1".to_string());

    let resp;

    {
        let res = Vec::new();
        con.read_to_end(&mut res);

        resp = String::from_utf8(&res).split("\n".to_string());
    }

    let mut name = "".to_string();
    let mut desc = "".to_string();
    let mut host = "".to_string();
    let mut files = "".to_string();
    let mut version = "".to_string();

    for i in resp {
        let data = i.substr(5, i.len() - 5);
        let key = i.substr(0, 4);

        if key == "name" {
            name = data;
        } else if key == "desc" {
            desc = data;
        } else if key == "host" {
            host = data;
        } else if key == "file" {
            files = data;
        } else if key == "vers" {
            version = data;
        }
    }

    Package {
        host: host,
        name: name,
        desc: desc,
        files: files.split(",".to_string()).collect::<Vec<_>>(),
        version: version,
    }
}
