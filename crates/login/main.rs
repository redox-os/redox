use std::hash::Hasher;
use std::io::{stdin, stdout, Write};
use std::process::Command;

pub struct Djb2 {
    state: u32,
}

impl Default for Djb2 {
    fn default() -> Djb2 {
        Djb2 {
            state: 5381,
        }
    }
}

impl Hasher for Djb2 {
    fn finish(&self) -> u64 {
        self.state as u64
    }

    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            // Update the state for each byte in the buffer.
            self.state = (self.state << 5).wrapping_add(self.state).wrapping_add(b as u32);
        }
    }
}


fn main() {
    print!("\x1Bc");
    loop {
        print!("LICENSE KEY (ENTER TO VIEW LICENSE): ");
        stdout().flush().unwrap();

        let mut buffer = String::new();
        stdin().read_line(&mut buffer).unwrap();

        if buffer == "\n" {
            Command::new("/bin/less").arg("/home/LICENSE.md").spawn().unwrap().wait().unwrap();
        } else {
            let mut hasher: Djb2 = Default::default();
            hasher.write(&buffer.as_bytes());
            let hash = hasher.finish();

            if hash == 0x6222EB0A {
                print!("\x1Bc");
                loop {
                    print!("DO YOU AGREE WITH THE LICENSE (Y/N/ENTER TO VIEW)? ");
                    stdout().flush().unwrap();

                    buffer = String::new();
                    stdin().read_line(&mut buffer).unwrap();

                    if buffer == "\n" {
                        Command::new("/bin/less").arg("/home/LICENSE.md").spawn().unwrap().wait().unwrap();
                    } else if buffer == "Y\n" || buffer == "y\n" {
                        Command::new("/bin/orbital").spawn().unwrap().wait().unwrap();
                        return;
                    } else {
                        println!("\x1BcYOU ARE NOT ALLOWED TO DISAGREE WITH THE LICENSE\n");
                    }
                }
            } else {
                println!("\x1BcTHE LICENSE KEY YOU PROVIDED IS NOT VALID");
                println!("PLEASE BUY A KEY AT WWW.REDOX-OS.ORG\n");
            }
        }
    }
}
