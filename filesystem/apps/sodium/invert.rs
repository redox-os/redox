use super::*;

impl Editor {
    pub fn invert_chars(&mut self, n: usize) {
        for _ in 0..n {
            let (x, y) = self.pos();
            let current = self.current();
            if let Some(c) = self.text[y].get_mut(x) {
                if let Some(cur) = current {
                    *c = invert::invert(cur);
                }
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
