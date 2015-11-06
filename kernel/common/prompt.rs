/// Input char to the prompt
pub fn run(c: char) {
    match c {
        '\0' => {},
        '\n' => {
            debug!("\n$ ");
        },
        ch => {
            debug!("{}", ch);
        },
    }

}
