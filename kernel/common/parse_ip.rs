use common::slice::GetSlice;

/// Get the port from a string (ip)
pub fn parse_port(string: &str) -> &str {
    let mut b = 1;
    for c in string.chars() {
        match c {
            '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => b += 1,
            _ => break,
        }
    }

    string.get_slice(string.find(':').map(|a| a + 1)..Some(b))
}

/// Get the host from a string (ip)
pub fn parse_host(string: &str) -> &str {
    string.get_slice(..string.find(|c| c == ':' || c == '/').map(|b| b + 1))
}
