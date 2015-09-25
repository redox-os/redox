use redox::*;

pub fn main(){
    console_title(&"Echo".to_string());
    loop {
        println!(readln!());
    }
}
