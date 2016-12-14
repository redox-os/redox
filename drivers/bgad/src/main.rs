use std::env;

fn main() {
    let mut args = env::args().skip(1);

    let mut name = args.next().expect("bgad: no name provided");
    name.push_str("_bga");

    let bar_str = args.next().expect("bgad: no address provided");
    let bar = usize::from_str_radix(&bar_str, 16).expect("bgad: failed to parse address");

    print!("{}", format!(" + BGA {} on: {:X}\n", name, bar));
}
