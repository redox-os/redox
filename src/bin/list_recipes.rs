use cookbook::recipe_find::list_recipes;
use std::path::Path;
use std::process::exit;
// use clap::Parser;

fn main() {

    let result = list_recipes( Path::new("recipes"));
    if result.is_err() {
        eprintln!("{}", result.err().unwrap());
        exit(2);
    } else if result.as_ref().unwrap().is_empty() {
        eprintln!("recipes not found");
        exit(1);
    } else {
        result.unwrap().iter().for_each(|recipe| println!("{}", recipe));
        exit(0);
    }
}
