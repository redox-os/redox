use cookbook::recipe_find::recipe_find;
use std::env::args;
use std::path::Path;
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
    let result = recipe_find(&args().last().unwrap(), Path::new("recipes"));
    if result.is_err() {
        eprintln!("{}", result.err().unwrap());
        exit(2);
    } else if result.as_ref().unwrap().is_none() {
        eprintln!("recipe {} not found", &args().last().unwrap());
        exit(1);
    } else {
        println!("{}", result.unwrap().unwrap().display());
        exit(0);
    }
}
