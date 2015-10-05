use redox::*;

//TODO: Find a way to clean up the to_string's
// Maybe make a Into trait, which is generic. Let rust's type interference do the work.
pub fn main() {
    console_title(&"Ox".to_string());

    let red = [255, 127, 127, 255];
    let green = [127, 255, 127, 255];
    let blue = [127, 127, 255, 255];

    while let Option::Some(line) = readln!() {
        let mut args: Vec<String> = Vec::new();
        for arg in line.split(" ".to_string()) {
            args.push(arg);
        }

        if let Option::Some(command) = args.get(0) {
            println!("# ".to_string() + line);

            if *command == "install".to_string() || *command == "i".to_string() {
                for i in 1..args.len() {
                    if let Option::Some(package) = args.get(i) {
                        print_color!("Install ".to_string() + package + "\n".to_string(), green);
                    }
                }
            } else if *command == "uninstall".to_string() || *command == "u".to_string() {
                for i in 1..args.len() {
                    if let Option::Some(package) = args.get(i) {
                        print_color!("Uninstall ".to_string() + package + "\n".to_string(), red);
                    }
                }
            }else{
                print_color!("Commands: install uninstall\n".to_string(), blue);
            }
        }
    }
}
