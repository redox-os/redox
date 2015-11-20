use super::*;
use redox::*;

impl Editor {
    /// Invoke a command in the prompt
    pub fn invoke(&mut self, cmd: String) {
        let mut split = cmd.split(' ');
        let base_cmd = split.nth(0).unwrap_or("");
        let sec_cmd = split.nth(0).unwrap_or("");

        match base_cmd {
            "set" => {
                self.status_bar.msg = match self.options.set(sec_cmd) {
                    Ok(()) => format!("Option set: {}", sec_cmd),
                    Err(()) => format!("Option does not exist: {}", sec_cmd),
                }
            }
            "unset" => {
                self.status_bar.msg = match self.options.unset(sec_cmd) {
                    Ok(()) => format!("Option unset: {}", sec_cmd),
                    Err(()) => format!("Option does not exist: {}", sec_cmd),
                }
            }
            "toggle" | "tog" => {
                self.status_bar.msg = match self.options.toggle(sec_cmd) {
                    Ok(()) => format!("Option toggled: {}", sec_cmd),
                    Err(()) => format!("Option does not exist: {}", sec_cmd),
                }
            }
            "get" => {
                self.status_bar.msg = match self.options.get(sec_cmd) {
                    Some(true) => format!("Option set: {}", sec_cmd),
                    Some(false) => format!("Option unset: {}", sec_cmd),
                    None => format!("Option does not exist: {}", sec_cmd),
                }
            }
            "o" | "open" => {
                self.status_bar.msg = match self.open(sec_cmd) {
                    OpenStatus::NotFound => format!("File {} could not be opened", sec_cmd),
                    OpenStatus::Ok => format!("File {} opened", sec_cmd),
                }
            }
            "help" => {
                self.open("/apps/sodium/help.txt");
            }
            c => {
                self.status_bar.msg = format!("Unknown command: {}", c);
            }
        }
    }
}
