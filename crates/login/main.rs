use std::env;
use std::fs::File;
use std::io::{stdin, stdout, Read, Write};
use std::process::Command;

fn main() {
    loop {
        print!("redox login: ");
        stdout().flush().unwrap();

        let mut buffer = String::new();
        stdin().read_line(&mut buffer).unwrap();

	if let Ok(mut motd) = File::open("/etc/motd") {
            let mut motd_string = String::new();
            if let Ok(_) = motd.read_to_string(&mut motd_string) {
                println!("{}", motd_string);
            }
        }

        env::set_current_dir("/home/").unwrap();

        let mut child = Command::new("/bin/sh").spawn().unwrap();
        child.wait().unwrap();
    }
}
