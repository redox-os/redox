use redox::*;
use common::*;
mod crypto;
use crypto::{Sha512, PubKey, RsaSignature};

/// Version
#[derive(Hash, Clone)]
pub enum Version {
    NewerThan(u64),
    OlderThan(u64),
    Between(u64, u64),
    Excatly(u64),
    Newest,
}

// TODO: Implement format

#[derive(Hash, Clone)]
/// A package developer
pub struct Developer {
    /// The name of the developer
    pub name: String,
    /// The public key of the developer
    pub key: PubKey,
}

#[derive(Hash, Clone)]
/// An installable package for Oxide
pub struct Package {
    /// The id of this package
    pub id: Id,
    /// Description
    pub desc: String,
    /// The developer of the package
    pub dev: Developer,
    /// The developer's signature of this package's content (tarball)
    pub dev_sign: RsaSignature,
    /// The signatures of this package
    pub sign: Vec<RsaSignature>,
    /// The files this package will create on the computer.
    pub files: Vec<String>,
    /// Dependencies of this package
    pub deps: Vec<Id>,
}

impl Package {
    /// Get content
    pub fn get_content(&self) -> Tarball {
        
    }

    /// Get package from string
    pub fn from_string(s: String) -> Option<Package> {

        // TODO

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

    /// Install package
    pub fn install(&self, col: &Collection) -> Result<(), InstallError> {
        // Install deps
        for d in self.deps {
            d.install();
        }

        // TODO install + add to local package list
    }

    /// Check validity
    pub fn check(&self, col: &Collection) -> TrustLevel {
        let con = self.get_content();

        for s in self.sign {
            if s.check(con) && col.keys.contains(s) {
                return TrustLevel::TrustedPackage
            } else if !s.check(con) {
                return TrustLevel::InvalidSignature
            }
        }

        if !self.dev_sign.check(con) {
            TrustLevel::InvalidSignature
        } else if !col.devs.contains(self.dev_sign) {
            TrustLevel::UntrustedSignature
        } else {
            TrustLevel::TrustedDev
        }
    }

}

/// Trust level
pub enum TrustLevel {
    /// 0
    InvalidSignature,
    /// 1
    UntrustedSignature,
    /// 2
    TrustedDeveloper,
    /// 3
    TrustedPackage,
}

impl TrustLevel {
    /// Is this package trusted?
    pub fn is_trusted(&self) -> bool {
        match self {
            &TrustLevel::TrustedDeveloper | TrustLevel::TrustedPackage => true,
            _ => false,
        }
    }
}

/// An error
pub enum PackageError {
    InvalidSyntax,
    InvalidSignature,
    UntrustedSignature,
    UntrustedDev,
    NotFound,
    E404,
    InfiniteDeps,
    Unknown,
}

#[derive(Hash, Clone)]
/// An package descriptor
pub struct Id {
    pub name: String,
    pub version: Version,
}

impl Id {
    pub fn to_string(&self) -> String {
        format!("{}-{}", self.name, self.version)
    }
}

/// Database of trusted developers
#[derive(Hash, Clone)]
pub struct DevDb {
    pub data: HashSet<Developer>,
}

/// Database of trusted keys
#[derive(Hash, Clone)]
pub struct KeyDb {
    pub data: HashSet<PubKey>,
}

/// An index of packages
#[derive(Hash, Clone)]
pub struct Index {
    /// Where the search queries can be send to
    pub host: String,
}

impl Index {
    /// Get a given package
    pub fn get(&self, id: Id) -> Result<Package, PackageError> {
        let con = File::open("tcp://".to_string() + self.host);

        con.write("GET /ox/".to_string() + id.to_string() + " HTTP/1.1".to_string());

        let res = Vec::new();
        con.read_to_end(&mut res);

        Package::from_string(String::from_utf8(&res))
    }
}

/// A collection of indexes, trusted keys, and trusted developers (all stored on the users
/// computer)
#[derive(Hash, Clone)]
pub struct Collection {
    /// Indexes
    pub index: Vec<Index>,
    /// The trusted devs
    pub devs: DevDb,
    /// The trusted keys
    pub keys: KeyDb,
    /// The installed packages
    pub installed: Vec<LocalPackage>,
}

/// A package installed locally
pub struct LocalPackage {
    /// Files it owns
    pub owns: Vec<String>,
    /// Is this package installed as root (i.e. isnt just installed as a dep for another package)?
    pub root: bool,
    /// The package
    pub package: Package,
}

impl LocalPackage {
    pub fn uninstall(&self) -> bool {

    }
}

impl Collection {
    /// Get a given package (guaranteed to be valid)
    pub fn get(&self, id: Id) -> Result<Package, PackageError> {
        for i in self.index {
            if let Ok(p) = i.get(id) {
                if p.check().is_trusted() {
                    return Ok(p);
                }
            }
        }
        None
    }
}


