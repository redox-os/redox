#![feature(unicode)]

extern crate walkdir;
extern crate rustc_unicode;

use walkdir::WalkDir;

use std::env;
use std::fs::{self, File};
use std::io::prelude::*;
use std::path::Path;

const START: &'static str = "@MANSTART";
const END: &'static str = "@MANEND";

fn main() {
    let mut args = env::args().skip(1);
    let source = args.next().unwrap_or(".".to_string());
    let output = args.next().unwrap_or("man".to_string());

    if ! Path::new(&output).is_dir() {
        fs::create_dir(&output).expect("Failed to create man directory");
    }

    for entry in WalkDir::new(&source).follow_links(true).into_iter().map(|x| x.expect("Failed to walk dir")).filter(|x| x.file_type().is_file()) {
        let mut string = String::new();

        File::open(entry.path()).expect("Could't open file.")
                                .read_to_string(&mut string)
                                .expect("Could't read file.");

        for i in string.split(START).skip(1) {
            let start_delimiter = i.find('{').expect("No opened { for MANSTART");
            let end_delimiter = i.find('}').expect("Unclosed '{' for MANSTART");
            let name = &i[start_delimiter + 1..end_delimiter];
            assert!(name.lines().count() == 1, "malformed manpage name");

            let man_page = &i[end_delimiter + 1..i.find(END).expect("Unclosed @MANSTART (use @MANEND)") + END.len()].trim();

            let mut string = String::with_capacity(man_page.len() + man_page.len() / 3);

            for i in man_page.lines().skip(1) {
                if i.find(END).is_none() {
                    string.push_str(i.trim_left_matches('\\')
                                     .trim_left_matches("// ")
                                     .trim_left_matches("//! ")
                                     .trim_left_matches("/// ")
                                     .trim_left_matches("->")
                                     .trim_left_matches("<!-"));
                    string.push('\n')
                }
            }

            println!("{} -> {}", entry.path().display(), output.clone() + "/" + name);
            let mut file = File::create(output.clone() + "/" + name).expect("Failed to create man page");
            file.write(&string.as_bytes()).expect("Failed to write man page");
        }
    }
}
