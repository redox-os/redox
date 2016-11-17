extern crate syscall;

use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn main() {
    let pid = unsafe { syscall::clone(syscall::CLONE_NEWNS).unwrap() };
    if pid == 0 {
        println!("Child Namespace:");
        let file = BufReader::new(File::open("sys:scheme").unwrap());
        for line in file.lines() {
            let line = line.unwrap();
            println!("{}", line);
        }
        println!("");
    } else {
        let mut status = 0;
        syscall::waitpid(pid, &mut status, 0).unwrap();

        println!("Parent Namespace:");
        let file = BufReader::new(File::open("sys:scheme").unwrap());
        for line in file.lines() {
            let line = line.unwrap();
            println!("{}", line);
        }
        println!("");
    }
}
