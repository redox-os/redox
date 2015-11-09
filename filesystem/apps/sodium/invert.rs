use super::*;

impl Editor {
    pub fn invert_chars(&mut self, n: usize) {
        let (x, y) = self.pos();
        for _ in 0..n {
            self.text[y][x] = invert::invert(self.current());
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
