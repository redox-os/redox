use collections::string::String;
use collections::vec::Vec;

/// Parse the path
pub fn parse_path(path: &str, cwd: Vec<String>) -> Vec<String> {
    // This method do also canonicalize the path
    let mut parts = if let Some('/') = path.chars().next() {
        Vec::new()
    } else {
        cwd
    };
    let mut new_part = true;
    let mut climb = false;
    let mut escape = false;

    let mut head = String::new();

    for c in path.chars() {
        if escape {
            head.push(c);
        } else {
            match c {
                '\\' => {
                    climb = false;
                    new_part = false;
                    escape = true;
                },
                '.' if new_part => {
                    climb = true;
                },
                '.' if climb => {
                    parts.pop();
                },
                '/' if !new_part => {
                    climb = false;
                    new_part = true;
                    parts.push(head.clone());
                    head.clear();
                },
                '/' => {},
                c => {
                    climb = false;
                    new_part = false;
                    head.push(c);
                },
            }
        }
    }

    if !head.is_empty() {
        parts.push(head);
    }

    parts
}
