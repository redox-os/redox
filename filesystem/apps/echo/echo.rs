use redox::*;

pub fn main() {
    console_title(&"Echo".to_string());
    while let Option::Some(line) = readln!() {
        println!(line);
    }
}
