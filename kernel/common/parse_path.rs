use collections::string::String;
use collections::vec::Vec;

/// Parse the path
pub fn parse_path(path: &str) -> Vec<String> {
    // This method do also canonicalize the path
    let mut parts = Vec::new();
    let mut new_part = true;
    let mut escape = false;

    let mut cur_part = String::new();

    for c in path.chars() {
        if escape {
            cur_part.push(c);
        } else {
            match c {
                '\\' => {
                    new_part = false;
                    escape = true;
                },
                '/' if !new_part => {
                    new_part = true;
                    parts.push(cur_part.clone());
                    cur_part.clear();
                },
                '/' => {},
                c => {
                    new_part = false;
                    cur_part.push(c);
                },
            }
        }
    }

    if !cur_part.is_empty() {
        parts.push(cur_part);
    }

    parts
}
