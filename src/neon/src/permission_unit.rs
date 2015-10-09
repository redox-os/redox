use str_match::str_match;

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
        // TODO
        self
        && str_match(self.param, lhs.param)
    }
}

