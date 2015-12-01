pub struct Options {
    pub highlight: bool,
    pub autoindent: bool,
    pub line_marker: bool,
}

impl Options {
    /// Create new default options
    pub fn new() -> Self {
        Options {
            highlight: true,
            autoindent: true,
            line_marker: true,
        }
    }

    /// Get the given option as a mutable reference
    pub fn get_mut(&mut self, name: &str) -> Option<&mut bool> {
        match name {
            "hightlight" | "hl" => Some(&mut self.highlight),
            "autoindent" | "ai" => Some(&mut self.autoindent),
            "line_marker" | "linemarker" | "linemark" | "lm" => Some(&mut self.line_marker),
            _ => None,
        }
    }

    /// Get a given option
    pub fn get(&self, name: &str) -> Option<bool> {
        match name {
            "hightlight" | "hl" => Some(self.highlight),
            "autoindent" | "ai" => Some(self.autoindent),
            "line_marker" | "linemarker" | "linemark" | "lm" => Some(self.line_marker),
            _ => None,
        }
    }

    /// Set a given option (mark it as active)
    pub fn set(&mut self, name: &str) -> Result<(), ()> {
        match self.get_mut(name) {
            Some(x) => {
                *x = true;
                Ok(())
            }
            None => Err(()),
        }
    }
    /// Unset a given option (mark it as inactive)
    pub fn unset(&mut self, name: &str) -> Result<(), ()> {
        match self.get_mut(name) {
            Some(x) => {
                *x = false;
                Ok(())
            }
            None => Err(()),
        }
    }
    /// Toggle a given option
    pub fn toggle(&mut self, name: &str) -> Result<(), ()> {
        match self.get_mut(name) {
            Some(x) => {
                *x = !*x;
                Ok(())
            }
            None => Err(()),
        }
    }

}
