/// Decompose the string pattern
pub fn decompose(rule: &str) -> Vec<String> {
    let mut res = vec![String::new()];
    let mut escape = false;
    for c in rule.chars() {
        let l = res.len();
        if escape {
            res[l - 1].push(c);
            escape = false;
        } else if c == '\\' {
            escape = true;
        } else if c == '*' {
            res.push(String::new());
        } else {
            res[l - 1].push(c);
        }
    }

    res
}

/// Check if a pattern matches a given string
pub fn str_match(rule: &str, other: &str) -> bool {
    let mut pos = 0;
    for i in decompose(rule) {
        match (&other[pos..]).find(&i) {
            Some(n) => pos = n,
            None => return false,
        }
    }

    true
}
