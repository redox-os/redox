use super::*;
use redox::*;

pub struct Options {
    pub highlight: bool,
    pub autoindent: bool,
}

impl Options {
    /// Create new default options
    pub fn new() -> Self {
        Options {
            highlight: true,
            autoindent: true,
        }
    }

    /// Get the given option as a mutable reference
    pub fn get_mut(&mut self, name: &str) -> Option<&mut bool> {
        match name {
            "hightlight" | "hl" => Some(&mut self.highlight),
            "autoindent" | "ai" => Some(&mut self.autoindent),
            _ => None,
        }
    }

    /// Get a given option
    pub fn get(&self, name: &str) -> Option<bool> {
        match name {
            "hightlight" | "hl" => Some(self.highlight),
            "autoindent" | "ai" => Some(self.autoindent),
            _ => None,
        }
    }

    pub fn set(&mut self, name: &str) -> Result<(), ()> {
        match self.get_mut(name) {
            Some(x) => {
                *x = true;
                Ok(())
            },
            None => Err(()),
        }
    }
    pub fn unset(&mut self, name: &str) -> Result<(), ()> {
        match self.get_mut(name) {
            Some(x) => {
                *x = false;
                Ok(())
            },
            None => Err(()),
        }
    }
    pub fn toggle(&mut self, name: &str) -> Result<(), ()> {
        match self.get_mut(name) {
            Some(x) => {
                *x = !*x;
                Ok(())
            },
            None => Err(()),
        }
    }

}

