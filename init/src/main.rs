use std::fs::File;
use std::io::{BufRead, BufReader};
use std::thread;

pub fn main() {
    let mut file = File::open("initfs:etc/init.rc").expect("failed to open init.rc");
    let mut reader = BufReader::new(file);

    for line in reader.lines() {
        println!("{}", line.expect("failed to read init.rc"));
    }

    loop {
        thread::yield_now();
    }
}
