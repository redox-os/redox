use std::thread;

pub fn main() {
    println!("Hello, World!");
    loop {
        thread::yield_now();
    }
}
