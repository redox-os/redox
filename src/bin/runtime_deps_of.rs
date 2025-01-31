use cookbook::recipe::CookRecipe;
use std::{env::args, process::ExitCode};

/// Same as `cookbook/src/bin/cook.rs`.
const DEP_DEPTH: usize = 16;

fn usage() {
    eprintln!("Usage: pkg_deps_of package1 [package2 ...]");
}

fn main() -> ExitCode {
    if args().len() < 2 {
        usage();
        return ExitCode::FAILURE;
    }

    let names = args().skip(1).collect::<Vec<String>>();
    let recipes = CookRecipe::new_recursive(&names, DEP_DEPTH, true).expect("recipe not found");

    for recipe in recipes {
        println!("{}", recipe.name);
    }

    ExitCode::SUCCESS
}
