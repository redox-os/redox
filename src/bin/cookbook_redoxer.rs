use std::env;

fn main() {
    let mut args: Vec<String> = env::args().collect();
    // Ensure all flags go to cargo
    if args.len() >= 2 {
        args.insert(2, "--".to_string());
    }
    redoxer::main(&args);
}
