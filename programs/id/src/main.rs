#![feature(question_mark)]

extern crate syscall;

use std::env;

pub fn main() {
    let uid = syscall::getuid().expect("id: failed to get UID");
    let gid = syscall::getgid().expect("id: failed to get GID");
    println!("uid={}({}) gid={}({})", uid, env::var("USER").unwrap_or(String::new()), gid, "");
}
