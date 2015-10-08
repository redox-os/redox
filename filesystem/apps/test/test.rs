/* For use with STD
use std::io;

macro_rules! readln {
    () => {
        {
            let mut line = String::new();
            match io::stdin().read_line(&mut line) {
                Ok(n) => Some(line.trim().to_string()),
                Err(e) => None
            }
        }
    };
}

fn console_title(title: &str){

}
*/

use redox::*;

pub fn main() {
    console_title("Test");

    while let Some(line) = readln!() {
        let mut args: Vec<String> = Vec::new();
        for arg in line.split(' ') {
            args.push(arg.to_string());
        }

        if let Some(command) = args.get(0) {
            println!("# {}", line);

            if command == "panic" {
                panic!("Test panic");
            } else {
                println!("Commands: panic");
            }
        }
    }
}
