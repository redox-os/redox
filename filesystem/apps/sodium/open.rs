use super::*;
use redox::prelude::v1::*;
use redox::fs::File;
use redox::io::Read;

pub enum OpenStatus {
    Ok,
    NotFound,
}

impl Editor {
    /// Open a file
    pub fn open(&mut self, path: &str) -> OpenStatus {
        self.status_bar.file = path.to_string();
        if let Some(mut file) = File::open(path) {
            let mut con = String::new();
            file.read_to_string(&mut con);

            self.text = con.lines()
                           .map(|x| x.chars().collect::<VecDeque<char>>())
                           .collect::<VecDeque<VecDeque<char>>>();

            OpenStatus::Ok
        } else {
            OpenStatus::NotFound
        }

    }
}
