extern crate syscall;

use std::fs::File;
use std::io::{BufRead, BufReader, Read};

pub fn main() {
    let pid = unsafe { syscall::clone(0).unwrap() };
    if pid == 0 {
        let rand = b"rand";
        syscall::setns(&[[rand.as_ptr() as usize, rand.len()]]).unwrap();

        println!("Child Namespace:");
        let file = BufReader::new(File::open("sys:scheme").unwrap());
        for line in file.lines() {
            let line = line.unwrap();
            println!("{}", line);
        }

        let mut rand = File::open("rand:").unwrap();

        let mut byte = [0];
        rand.read(&mut byte).unwrap();

        println!("Rand: {}", byte[0]);
    } else {
        let mut status = 0;
        syscall::waitpid(pid, &mut status, 0).unwrap();

        println!("Parent Namespace:");
        let file = BufReader::new(File::open("sys:scheme").unwrap());
        for line in file.lines() {
            let line = line.unwrap();
            println!("{}", line);
        }
    }
}
