
/// Get the port from a string (ip)
pub fn parse_port(string: &str) -> &str {
    let a = match string.find(':') {
        Some(pos) => pos + 1,
        None => 0,
    };

    let mut b = 0;
    for c in string.chars().skip(a) {
        match c {
            '0' | '1' |
            '2' | '3' |
            '4' | '5' |
            '6' | '7' |
            '8' | '9' => b += 1,
            _ => break,
        }
    }

    &string[a..b + 1]
}

/// Get the host from a string (ip)
pub fn parse_host(string: &str) -> &str {
    let pos = match string.find(|c| c == ':' || c == '/') {
        Some(pos) => pos + 1,
        None => string.len(),
    };

    &string[..pos]
}
