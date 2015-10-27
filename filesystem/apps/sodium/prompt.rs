use super::*;
use redox::*;

impl Editor {
    pub fn invoke(&mut self, cmd: String) {
        let mut split = cmd.split(' ');
        let base_cmd = split.nth(0).unwrap_or("");
        let sec_cmd = split.nth(0).unwrap_or("");

        match base_cmd {
            "set" => {
                self.options.set(sec_cmd);
            },
            "unset" => {
                self.options.unset(sec_cmd);
            },
            "toggle" | "tog" => {
                self.options.toggle(sec_cmd);
            },
            _ => {},
        }
    }
}
