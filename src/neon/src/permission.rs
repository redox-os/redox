use permission_unit::PermissionUnit;

/// A "component" of a permission, i.e. except/and permission unit
#[derive(Clone)]
pub enum Component {
    And(PermissionUnit),
    Except(PermissionUnit),
}
impl Component {
    /// Get the permission unit of the component
    pub fn unit(&self) -> &PermissionUnit {
        match *self {
            Component::And(ref x) => x,
            Component::Except(ref x) => x,
        }
    }
}

/// A permission rule
#[derive(Clone)]
pub struct Permission {
    /// The components
    pub components: Vec<Component>,
}

impl Permission {
    /// Create a permission from a string
    pub fn from_str(string: &str) -> Permission {
        
    }

    /// Test if this permission unit is allowed
    pub fn test(&self, unit: PermissionUnit) -> bool {
        let mut read = true;
        let mut write = true;
        let mut read_foc = true;
        let mut write_foc = true;

        for i in self.components.clone() {
            if i.unit().applies(&unit) {
                match i {
                    Component::And(ref x) => {
                        if x.read() {
                            read = true;
                        }
                        if x.write() {
                            write = true;
                        }
                        if x.read_foc() {
                            read_foc = true;
                        }
                        if x.write_foc() {
                            write_foc = true;
                        }
                    },

                    Component::Except(ref x) => {
                        if x.read() {
                            read = false;
                        }
                        if x.write() {
                            write = false;
                        }
                        if x.read_foc() {
                            read_foc = false;
                        }
                        if x.write_foc() {
                            write_foc = false;
                        }
                    }
                }
            }
        }

        (!read || unit.read())
            && (!write || unit.write())
            && (!read_foc || unit.read_foc())
            && (!write_foc || unit.write_foc())
    }
}
