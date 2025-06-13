use cookbook::recipe_find::recipe_find;
use std::env::args;
use std::process::exit;
// use clap::Parser;

fn usage() {
    println!("Usage: find_recipe recipe_name");
}
fn main() {
    if args().len() != 2 {
        usage();
        exit(2);
    }
    let recipe_name = &args().last().unwrap();
    match recipe_find(recipe_name) {
        Some(path) => {
            println!("{}", path.display());
            exit(0);
        },
        None => {
        eprintln!("recipe {} not found", recipe_name);
        exit(1);
        }
    }
}
