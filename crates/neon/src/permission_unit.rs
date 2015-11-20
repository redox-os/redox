use str_match::str_match;

#[derive(Clone)]
pub struct PermissionUnit {
    /// Permission to read
    pub read: bool,
    /// Permission to write
    pub write: bool,
    /// Permission to read when focused
    pub read_foc: bool,
    /// Permission to write when focused
    pub write_foc: bool,
    /// Was a unexpected char found?
    pub error: bool,

    /// The parameter provided
    pub param: String,
}

impl PermissionUnit {
    /// Create a permission unit from a string
    pub fn from_str(unit: &str) -> PermissionUnit {
        let mut read = false;
        let mut write = false;
        let mut read_foc = false;
        let mut write_foc = false;
        let mut error = false;
        let mut param_col = false;
        let mut param = String::new();

        for c in unit.chars() {
            if param_col {
                param.push(c);
            } else {
                match c {
                    'r' => read = true,
                    'w' => write = true,
                    'R' => read_foc = true,
                    'W' => write_foc = true,
                    '=' => param_col = true,
                    _ => error = true,
                }
            }
        }

        PermissionUnit {
            read: read,
            write: write,
            read_foc: read_foc,
            write_foc: write_foc,
            error: error,
            param: param,
        }
    }

    /// Does this permission unit implies another one
    pub fn implies(&self, lhs: PermissionUnit) -> bool {
        // Self implies lhs
        (!self.read || lhs.read) && (!self.write || lhs.write) &&
        (!self.read_foc || lhs.read_foc) && (!self.write_foc || lhs.write_foc) &&
        str_match(&self.param, &lhs.param)
    }

    /// Does this permission unit apply to another one
    pub fn applies(&self, other: &PermissionUnit) -> bool {
        str_match(&self.param, &other.param)
    }

    /// Is this permission unit readable?
    pub fn read(&self) -> bool {
        self.read
    }
    /// Is this permission unit writable?
    pub fn write(&self) -> bool {
        self.write
    }
    /// Is this permission unit readable when focused?
    pub fn read_foc(&self) -> bool {
        self.read || self.read_foc
    }
    /// Is this permission unit writable when focused?
    pub fn write_foc(&self) -> bool {
        self.write || self.write_foc
    }

    /// Is this given permission unit readble following the permission unit?
    pub fn is_readable(&self, other: PermissionUnit) -> bool {
        (!self.read || other.read) && str_match(&self.param, &other.param)
    }
    /// Readble on focus?
    pub fn is_readable_foc(&self, other: PermissionUnit) -> bool {
        (!self.read_foc() || other.read_foc()) && str_match(&self.param, &other.param)
    }
    /// Is this unit writable?
    pub fn is_writeable(&self, other: PermissionUnit) -> bool {
        (!self.write || other.write) && str_match(&self.param, &other.param)
    }
    /// Writable on focus?
    pub fn is_writeable_foc(&self, other: PermissionUnit) -> bool {
        (!self.write_foc() || other.write_foc()) && str_match(&self.param, &other.param)
    }
}
