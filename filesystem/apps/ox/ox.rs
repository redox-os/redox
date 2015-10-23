use redox::*;

pub fn main() {
    console_title("Ox");

    let red = Color::rgba(255, 127, 127, 255);
    let green = Color::rgba(127, 255, 127, 255);
    let blue = Color::rgba(127, 127, 255, 255);

    println!("Type help for a command list");
    while let Some(line) = readln!() {
        let mut args: Vec<String> = Vec::new();
        for arg in line.split(' ') {
            args.push(arg.to_string());
        }

        if let Some(command) = args.get(0) {
            println!("# {}", line);

            if command == "install" || command == "i" {
                for i in 1..args.len() {
                    if let Some(package) = args.get(i) {
                        println_color!(green, "Install {}", package);
                    }
                }
            } else if command == "uninstall" || command == "u" {
                for i in 1..args.len() {
                    if let Some(package) = args.get(i) {
                        println_color!(red, "Uninstall {}", package);
                    }
                }
            } else {
                print_color!(blue, "Commands: install uninstall");
            }
        }
    }
}
