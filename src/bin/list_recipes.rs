use std::process::exit;

use pkg::recipes;

fn main() {
    let print_short = std::env::args()
        .nth(1)
        .map_or(false, |a| a == "-s" || a == "--short");

    let result = recipes::list("");
    if result.is_empty() {
        eprintln!("recipes not found");
        exit(1);
    } else {
        for path in result {
            let Some(file_name) = path.file_name() else {
                continue;
            };

            if print_short {
                println!("{}", file_name.to_string_lossy());
            } else {
                println!("{}", path.to_string_lossy());
            }
        }
        exit(0);
    }
}
