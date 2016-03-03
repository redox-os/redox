#![feature(unicode)]

extern crate walkdir;
extern crate rustc_unicode;

use walkdir::WalkDir;
use std::fs::File;
use std::io::prelude::*;

fn main() {
    for entry in WalkDir::new(".").follow_links(true).into_iter().map(|x| x.expect("Couldn't walk dir.")).filter(|x| x.file_type().is_file()) {
        let mut string = String::new();

        File::open(entry.path()).expect("Could't open file.")
                                .read_to_string(&mut string)
                                .expect("Could't read file.");

        for i in string.split("@MANSTART{").skip(1) {
            let end_delimiter = i.find('}').expect("Unclosed '{'");
            let name = &i[..end_delimiter];
            assert!(name.lines().count() == 1, "malformed manpage name");

            let mut man_page = &i[end_delimiter + 1..i.find("@MANEND")
                                                      .expect("Unclosed @MANSTART (use @MANEND).")].trim();

            let mut string = String::with_capacity(man_page.len() + man_page.len() / 3);

            for i in man_page.lines() {
                string.push_str(i.trim_left_matches('\\')
                                 .trim_left_matches("// ")
                                 .trim_left_matches("//! ")
                                 .trim_left_matches("/// ")
                                 .trim_left_matches("->")
                                 .trim_left_matches("<!-"));
                string.push('\n')
            }

            let mut file = File::create("man/".to_owned() + name).expect("Couldn't create man page.");
            file.write(&string.as_bytes()[1..string.len() - 1]).expect("Couldn't write man page.");
        }
    }
}
