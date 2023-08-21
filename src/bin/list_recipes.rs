use cookbook::recipe_find::list_recipes;
use std::path::Path;
use std::process::exit;
// use clap::Parser;

fn main() {
    let result = list_recipes( Path::new("recipes"));

    match result {
        Ok(result) => {
            if result.is_empty() {
                eprintln!("recipes not found");
                exit(1);
            } else {
                result.iter().for_each(|recipe| println!("{recipe}"));
                exit(0);
            }
        }
        Err(error) => {
            eprintln!("{error}");
            exit(2);
        }
    }
}
