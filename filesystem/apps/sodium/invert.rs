use super::*;

impl Editor {
    pub fn invert_chars(&mut self, n: usize) {
        for _ in 0..n {
            let (x, y) = self.pos();
            let cur = self.current();
            if let Some(c) = self.text[y].get_mut(x) {
                *c = invert::invert(cur);
            }
            if let Some(m) = self.next(1) {
                self.goto(m);
            }
        }
    }
}

pub fn invert(c: char) -> char {
    match c {
        '<' => '>',
        '>' => '<',
        '&' => '|',
        '*' => '/',
        '(' => ')',
        ')' => '(',
        '+' => '-',
        '-' => '+',
        ';' => ':',
        ':' => ';',
        '\\' => '/',
        '/' => '\\',
        ',' => '.',
        '.' => ',',
        '\'' => '"',
        '"' => '\'',
        '[' => ']',
        ']' => '[',
        '{' => '}',
        '}' => '{',
        '!' => '?',
        '?' => '!',
        a => if a.is_lowercase() {
            a.to_uppercase().next().unwrap_or('?')
        } else {
            a.to_lowercase().next().unwrap_or('?')
        },
    }
}
