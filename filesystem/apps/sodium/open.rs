use editor::Editor;
use std::fs::File;
use std::io::Read;
use std::collections::VecDeque;

pub enum OpenStatus {
    Ok,
    NotFound,
}

impl Editor {
    /// Open a file
    pub fn open(&mut self, path: &str) -> OpenStatus {
        self.status_bar.file = path.to_string();
        if let Some(mut file) = File::open(path).ok() {
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
