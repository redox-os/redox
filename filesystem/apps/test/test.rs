use redox::*;

pub fn main() {
    console_title("Test");

    let red = [255, 127, 127, 255];
    let green = [127, 255, 127, 255];
    let blue = [127, 127, 255, 255];

    while let Option::Some(line) = readln!() {
        let mut args: Vec<String> = Vec::new();
        for arg in line.split(' ') {
            args.push(arg.to_string());
        }

        if let Option::Some(command) = args.get(0) {
            println!("# {}", line);

            if command == "panic" {
                panic!("Test panic");
            } else {
                print_color!(blue, "Commands: panic");
            }
        }
    }
}
