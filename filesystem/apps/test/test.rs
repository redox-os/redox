use std::{io, fs};

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

fn console_title(title: &str) {

}

#[no_mangle]
pub fn main() {
    console_title("Test");

    println!("Type help for a command list");
    while let Some(line) = readln!() {
        let args: Vec<String> = line.split(' ').map(|arg| arg.to_string()).collect();

        if let Some(command) = args.get(0) {
            println!("# {}", line);

            match &command[..]
            {
                "panic" => panic!("Test panic"),
                "ls" => {
                    // TODO: when libredox is completed
                    //fs::read_dir("/").unwrap().map(|dir| println!("{}", dir));
                }
                _ => println!("Commands: panic"),
            }
        }
    }
}
