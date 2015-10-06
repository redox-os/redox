use core::clone::Clone;
use core::cmp::PartialEq;
use core::iter::Iterator;
use core::ops::Add;
use core::ops::Index;
use core::option::Option;
use core::ptr;
use core::slice::SliceExt;
use core::str::StrExt;

use common::debug::*;
use common::vec::*;

use syscall::call::*;

/// A trait for types that can be converted to `String`
pub trait ToString {
    fn to_string(&self) -> String;
}

impl ToString for &'static str {
    /// Convert the type to `String`
    fn to_string(&self) -> String {
        String::from_str(self)
    }
}

/// An iterator over unicode characters
pub struct Chars<'a> {
    string: &'a String,
    offset: usize,
}

impl <'a> Iterator for Chars<'a> {
    type Item = char;
    fn next(&mut self) -> Option<Self::Item> {
        if self.offset < self.string.len() {
            let ret = Option::Some(self.string[self.offset]);
            self.offset += 1;
            ret
        } else {
            Option::None
        }
    }
}

/// A split
pub struct Split<'a> {
    string: &'a String,
    offset: usize,
    seperator: String,
}

impl <'a> Iterator for Split<'a> {
    type Item = String;
    fn next(&mut self) -> Option<Self::Item> {
        if self.offset < self.string.len() {
            let start = self.offset;
            let mut len = 0;
            for i in start..self.string.len() {
                if self.seperator == self.string.substr(i, self.seperator.len()) {
                    self.offset += self.seperator.len();
                    break;
                } else {
                    self.offset += 1;
                    len += 1;
                }
            }
            Option::Some(self.string.substr(start, len))
        } else {
            Option::None
        }
    }
}

/// A heap allocated, owned string.
pub struct String {
    pub vec: Vec<char>,
}

impl String {
    /// Create a new empty `String`
    pub fn new() -> Self {
        String { vec: Vec::new() }
    }

    // TODO FromStr trait
    /// Convert a string literal to a `String`
    pub fn from_str(s: &str) -> Self {
        let mut vec: Vec<char> = Vec::new();

        for c in s.chars() {
            vec.push(c);
        }

        String { vec: vec }
    }

    /// Convert a c-style string slice to a String
    pub fn from_c_slice(s: &[u8]) -> Self {
        let mut vec: Vec<char> = Vec::new();

        for b in s {
            let c = *b as char;
            if c == '\0' {
                break;
            }
            vec.push(c);
        }

        String { vec: vec }
    }

    /// Convert a utf8 vector to a string
    // Why &Vec?
    pub fn from_utf8(utf_vec: &Vec<u8>) -> Self {
        let mut vec: Vec<char> = Vec::new();

        //TODO: better UTF support
        for b in utf_vec.iter() {
            let c = *b as char;
            if c == '\0' {
                break;
            }
            vec.push(c);
        }

        String { vec: vec }
    }

    /// Convert a C-style string literal to a `String`
    pub unsafe fn from_c_str(s: *const u8) -> Self {
        let mut vec: Vec<char> = Vec::new();

        let mut i = 0;
        loop {
            let c = ptr::read(s.offset(i)) as char;
            if c == '\0' {
                break;
            }
            vec.push(c);
            i += 1;
        }

        String { vec: vec }
    }

    /// Convert an integer to a String using a given radix
    pub fn from_num_radix(num: usize, radix: usize) -> Self {
        if radix == 0 {
            return String::new();
        }

        let mut vec: Vec<char> = Vec::new();

        let mut digit_num = num;
        loop {
            let mut digit = (digit_num % radix) as u8;
            if digit > 9 {
                digit += 'A' as u8 - 10;
            } else {
                digit += '0' as u8;
            }

            vec.insert(0, digit as char);

            if digit_num < radix {
                break;
            }

            digit_num /= radix;
        }

        String { vec: vec }
    }

    /// Convert a signed integer to a String
    // TODO: Consider using `int` instead of `num`
    pub fn from_num_radix_signed(num: isize, radix: usize) -> Self {
        if num >= 0 {
            String::from_num_radix(num as usize, radix)
        } else {
            "-".to_string() + String::from_num_radix((-num) as usize, radix)
        }
    }

    /// Convert a `char` to a string
    pub fn from_char(c: char) -> Self {
        if c == '\0' {
            return String::new();
        }

        let mut vec: Vec<char> = Vec::new();
        vec.push(c);

        String { vec: vec }
    }

    /// Convert an unsigned integer to a `String` in base 10
    pub fn from_num(num: usize) -> Self {
        String::from_num_radix(num, 10)
    }

    /// Convert a signed int to a `String` in base 10
    pub fn from_num_signed(num: isize) -> Self {
        String::from_num_radix_signed(num, 10)
    }

    /// Get a substring
    // TODO: Consider to use a string slice
    pub fn substr(&self, start: usize, len: usize) -> Self {
        let mut i = start;
        if i > self.len() {
            i = self.len();
        }

        let mut j = i + len;
        if j > self.len() {
            j = self.len();
        }

        let mut vec: Vec<char> = Vec::new();

        for k in i..j {
            vec.push(self[k]);
        }

        String { vec: vec }
    }

    /// Find the index of a substring in a string
    pub fn find(&self, other: Self) -> Option<usize> {
        if self.len() >= other.len() {
            for i in 0..self.len() + 1 - other.len() {
                if self.substr(i, other.len()) == other {
                    return Option::Some(i);
                }
            }
        }
        Option::None
    }

    /// Check if the string starts with a given string
    pub fn starts_with(&self, other: Self) -> bool {
        if self.len() >= other.len() {
            // FIXME: This is inefficient
            self.substr(0, other.len()) == other
        } else {
            false
        }
    }

    /// Check if a string ends with another string
    pub fn ends_with(&self, other: Self) -> bool {
        if self.len() >= other.len() {
            // FIXME: Inefficient
            self.substr(self.len() - other.len(), other.len()) == other
        } else {
            false
        }
    }


    /// Get the length of the string
    pub fn len(&self) -> usize {
        self.vec.len()
    }

    /// Get a iterator over the chars of the string
    pub fn chars(&self) -> Chars {
        Chars {
            string: &self,
            offset: 0,
        }
    }

    /// Get a iterator of the splits of the string (by a seperator)
    pub fn split(&self, seperator: Self) -> Split {
        Split {
            string: &self,
            offset: 0,
            seperator: seperator,
        }
    }

    /// Convert the string to UTF-8
    pub fn to_utf8(&self) -> Vec<u8> {
        let mut vec: Vec<u8> = Vec::new();

        for c in self.chars() {
            let u = c as usize;
            if u < 0x80 {
                vec.push(u as u8);
            } else if u < 0x800 {
                vec.push(0b11000000 | ((u >> 6) as u8 & 0b00011111));
                vec.push(0b10000000 | (u as u8 & 0b00111111));
            } else if u < 0x10000 {
                vec.push(0b11100000 | ((u >> 12) as u8 & 0b00001111));
                vec.push(0b10000000 | ((u >> 6) as u8 & 0b00111111));
                vec.push(0b10000000 | (u as u8 & 0b00111111));
            } else {
                d("Unhandled to_utf8 code ");
                dh(u);
                dl();
                unsafe {
                    dh(self.vec.as_ptr() as usize);
                }
                d(" ");
                dd(self.vec.len());
                d(" to ");
                unsafe {
                    dh(vec.as_ptr() as usize);
                }
                d(" ");
                dd(vec.len());
                dl();
                break;
            }
        }

        vec
    }

    /// Convert the string to a C-style string
    pub unsafe fn to_c_str(&self) -> *const u8 {
        let length = self.len() + 1;

        let data = sys_alloc(length) as *mut u8;

        for i in 0..self.len() {
            ptr::write(data.offset(i as isize), self[i] as u8);
        }
        ptr::write(data.offset(self.len() as isize), 0);

        data
    }

    /// Parse the string to a integer using a given radix
    pub fn to_num_radix(&self, radix: usize) -> usize {
        if radix == 0 {
            return 0;
        }

        let mut num = 0;
        for c in self.chars() {
            let digit;
            if c >= '0' && c <= '9' {
                digit = c as usize - '0' as usize
            } else if c >= 'A' && c <= 'Z' {
                digit = c as usize - 'A' as usize + 10
            } else if c >= 'a' && c <= 'z' {
                digit = c as usize - 'a' as usize + 10
            } else {
                break;
            }

            if digit >= radix {
                break;
            }

            num *= radix;
            num += digit;
        }

        num
    }

    /// Parse the string as a signed integer using a given radix
    pub fn to_num_radix_signed(&self, radix: usize) -> isize {
        if self[0] == '-' {
            -(self.substr(1, self.len() - 1).to_num_radix(radix) as isize)
        } else {
            self.to_num_radix(radix) as isize
        }
    }

    /// Parse it as a unsigned integer in base 10
    pub fn to_num(&self) -> usize {
        self.to_num_radix(10)
    }

    /// Parse it as a signed integer in base 10
    pub fn to_num_signed(&self) -> isize {
        self.to_num_radix_signed(10)
    }

    pub fn d(&self) {
        for c in self.chars() {
            dc(c);
        }
    }
}

static NULL_CHAR: char = '\0';

impl Index<usize> for String {
    type Output = char;
    fn index<'a>(&'a self, i: usize) -> &'a Self::Output {
        match self.vec.get(i) {
            Option::Some(c) => c,
            Option::None => &NULL_CHAR,
        }
    }
}

impl PartialEq for String {
    fn eq(&self, other: &Self) -> bool {
        if self.len() == other.len() {
            for i in 0..self.len() {
                if self[i] != other[i] {
                    return false;
                }
            }

            true
        } else {
            false
        }
    }
}

impl Clone for String {
    fn clone(&self) -> Self {
        self.substr(0, self.len())
    }
}

impl<'a> Add<&'a String> for String {
    type Output = String;
    fn add(mut self, other: &'a Self) -> Self {
        self.vec.push_all(&other.vec);
        self
    }
}

impl<'a> Add<&'a mut String> for String {
    type Output = String;
    fn add(mut self, other: &'a mut Self) -> Self {
        self.vec.push_all(&other.vec);
        self
    }
}

impl Add for String {
    type Output = String;
    fn add(mut self, other: Self) -> Self {
        self.vec.push_all(&other.vec);
        self
    }
}

impl<'a> Add<&'a str> for String {
    type Output = String;
    fn add(self, other: &'a str) -> Self {
        self + String::from_str(other)
    }
}

impl Add<char> for String {
    type Output = String;
    fn add(mut self, other: char) -> Self {
        self.vec.push(other);
        self
    }
}

impl Add<usize> for String {
    type Output = String;
    fn add(self, other: usize) -> Self {
        self + String::from_num(other)
    }
}

impl Add<isize> for String {
    type Output = String;
    fn add(self, other: isize) -> Self {
        self + String::from_num_signed(other)
    }
}
