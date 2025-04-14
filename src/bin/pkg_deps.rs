use cookbook::package::StageToml;
use std::{env::args, process::ExitCode};

/// Same as `cookbook/src/bin/cook.rs`.
const DEP_DEPTH: usize = 16;

fn usage() {
    eprintln!("Usage: pkg_deps [package1 package2 ...]");
}

fn main() -> ExitCode {
    let names = args().skip(1).collect::<Vec<String>>();
    let packages = StageToml::new_recursive(&names, DEP_DEPTH).expect("package not found");

    for package in packages {
        println!("{}", package.name);
    }

    ExitCode::SUCCESS
}
