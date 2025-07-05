use std::{env::args, process::ExitCode};

use pkg::package::Package;

use cookbook::WALK_DEPTH;

fn main() -> ExitCode {
    let names = args().skip(1).collect::<Vec<String>>();
    // TODO: Ugly vec
    let names: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
    let packages = Package::new_recursive(&names, WALK_DEPTH).expect("package not found");

    for package in packages {
        println!("{}", package.name);
    }

    ExitCode::SUCCESS
}
