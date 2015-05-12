use core::iter::Iterator;
use core::mem::size_of;
use core::ops::Add;
use core::ops::Drop;
use core::ops::Index;
use core::option::Option;
use core::slice::SliceExt;
use core::str::StrExt;

use common::debug::*;
use common::memory::*;

pub trait ToString {
    fn to_string(&self) -> String;
}

impl ToString for &'static str {
    fn to_string(&self) -> String {
        String::from_str(self)
    }
}

struct Chars<'a> {
    string: &'a String,
    offset: usize
}

impl <'a> Iterator for Chars<'a> {
    type Item = char;
    fn next(&mut self) -> Option<Self::Item>{
        if self.offset < self.string.len() {
            let ret = Option::Some(self.string[self.offset]);
            self.offset += 1;
            return ret;
        }else{
            return Option::None;
        }
    }
}

struct Split<'a> {
    string: &'a String,
    offset: usize,
    seperator: String
}

impl <'a> Iterator for Split<'a> {
    type Item = String;
    fn next(&mut self) -> Option<Self::Item>{
        if self.offset < self.string.len() {
            let start = self.offset;
            let mut len = 0;
            for i in start..self.string.len(){
                if self.seperator.equals(self.string.substr(i, self.seperator.len())){
                    self.offset += self.seperator.len();
                    break;
                }else{
                    self.offset += 1;
                    len += 1;
                }
            }
            return Option::Some(self.string.substr(start, len));
        }else{
            return Option::None;
        }
    }
}

pub struct String {
    data: *const char,
    length: usize
}

impl String {
    pub fn new() -> String {
        String {
            data: 0 as *const char,
            length: 0
        }
    }

    // TODO FromStr trait
    pub fn from_str(s: &str) -> String {
        let length = s.chars().count();

        if length == 0 {
            return String::new();
        }

        let data = alloc(length * size_of::<char>());

        let mut i = 0;
        for c in s.chars() {
            unsafe {
                *((data + i * size_of::<char>()) as *mut char) = c;
            }
            i += 1;
        }

        String {
            data: data as *const char,
            length: length
        }
    }

    pub fn from_c_slice(s: &[u8]) -> String {
        let mut length = 0;
        for c in s {
            if *c == 0 {
                break;
            }
            length += 1;
        }

        if length == 0 {
            return String::new();
        }

        let data = alloc(length * size_of::<char>());

        let mut i = 0;
        for c in s {
            if i >= length {
                break;
            }
            unsafe {
                *((data + i * size_of::<char>()) as *mut char) = *c as char;
            }
            i += 1;
        }

        String {
            data: data as *const char,
            length: length
        }
    }

    pub unsafe fn from_c_str(s: *const u8) -> String {
        let mut length = 0;
        loop {
            if *(((s as usize) + length) as *const u8) == 0 {
                break;
            }
            length += 1;
        }

        if length == 0 {
            return String::new();
        }

        let data = alloc(length * size_of::<char>());

        for i in 0..length {
            *((data + i * size_of::<char>()) as *mut char) = *(((s as usize) + i) as *const u8) as char;
        }

        String {
            data: data as *const char,
            length: length
        }
    }

    pub fn from_num_radix(num: usize, radix: usize) -> String {
        if radix == 0 {
            return String::new();
        }

        let mut length = 1;
        let mut length_num = num;
        while length_num >= radix {
            length_num /= radix;
            length += 1;
        }

        let data = alloc(length * 4);

        let mut digit_num = num;
        for i in 0..length {
            let mut digit = (digit_num % radix) as u8;
            if digit > 9 {
                digit += 'A' as u8 - 10;
            }else{
                digit += '0' as u8;
            }

            unsafe {
                *((data + (length - 1 - i) * size_of::<char>()) as *mut char) = digit as char;
            }
            digit_num /= radix;
        }

        String {
            data: data as *const char,
            length: length
        }
    }

    pub fn from_char(c: char) -> String {
        if c == '\0' {
            return String::new();
        }

        let data = alloc(size_of::<char>());
        unsafe {
            *(data as *mut char) = c;
        }

        String {
            data: data as *const char,
            length: 1
        }
    }

    pub fn from_num(num: usize) -> String {
        String::from_num_radix(num, 10)
    }

    pub fn substr(&self, start: usize, len: usize) -> String {
        let mut i = start;
        if i > self.len() {
            i = self.len();
        }

        let mut j = i + len;
        if j > self.len() {
            j = self.len();
        }

        let length = j - i;
        if length == 0 {
            return String::new();
        }

        let data = alloc(length * 4);

        for k in i..j {
            unsafe {
                *((data + (k - i)*4) as *mut char) = *(((self.data as usize) + k*4) as *const char);
            }
        }

        String {
            data: data as *const char,
            length: length
        }
    }

    pub fn clone(&self) -> String {
        return self.substr(0, self.len());
    }

    pub fn equals(&self, other: String) -> bool {
        if self.len() == other.len() {
            for i in 0..self.len() {
                if self[i] != other[i] {
                    return false;
                }
            }

            return true;
        }else{
            return false;
        }
    }

    pub fn starts_with(&self, other: String) -> bool {
        if self.len() >= other.len() {
            return self.substr(0, other.len()).equals(other);
        }else{
            return false;
        }
    }

    pub fn ends_with(&self, other: String) -> bool {
        if self.len() >= other.len() {
            return self.substr(self.len() - other.len(), other.len()).equals(other);
        }else{
            return false;
        }
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn chars(&self) -> Chars {
        Chars {
            string: &self,
            offset: 0
        }
    }

    pub fn split(&self, seperator: String) -> Split {
        Split {
            string: &self,
            offset: 0,
            seperator: seperator
        }
    }

    pub unsafe fn to_c_str(&self) -> *const u8 {
        let length = self.len() + 1;

        let data = alloc(length);

        for i in 0..self.len() {
            *((data + i) as *mut u8) = *(((self.data as usize) + i * size_of::<char>()) as *const char) as u8;
        }
        *((data + self.len()) as *mut u8) = 0;

        data as *const u8
    }

    pub fn to_num_radix(&self, radix: usize) -> usize {
        if radix == 0 {
            return 0;
        }

        let mut num = 0;
        for c in self.chars(){
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

    pub fn to_num(&self) -> usize {
        self.to_num_radix(10)
    }

    pub fn d(&self){
        for c in self.chars() {
            dc(c);
        }
    }
}

static NULL_CHAR: char = '\0';

impl Index<usize> for String {
    type Output = char;
    fn index<'a>(&'a self, i: usize) -> &'a Self::Output {
        if i >= self.len() {
            // Failure condition
            return &NULL_CHAR;
        }else{
            unsafe{
                return &*(((self.data as usize) + i * size_of::<char>()) as *const char);
            }
        }
    }
}

impl Drop for String {
    fn drop(&mut self){
        unalloc(self.data as usize);
        self.data = 0 as *const char;
        self.length = 0;
    }
}

impl Add for String {
    type Output = String;
    fn add(self, other: String) -> String {
        let length = self.length + other.length;

        if length == 0 {
            return String::new();
        }

        let data = alloc(length * 4);

        let mut i = 0;
        for c in self.chars() {
            unsafe {
                *((data + i * size_of::<char>()) as *mut char) = c;
            }
            i += 1;
        }
        for c in other.chars() {
            unsafe {
                *((data + i * size_of::<char>()) as *mut char) = c;
            }
            i += 1;
        }

        String {
            data: data as *const char,
            length: length
        }
    }
}

impl<'a> Add<&'a String> for String {
    type Output = String;
    fn add(self, other: &'a String) -> String {
        self + other.clone()
    }
}


impl<'a> Add<&'a str> for String {
    type Output = String;
    fn add(self, other: &'a str) -> String {
        self + String::from_str(other)
    }
}

impl Add<char> for String {
    type Output = String;
    fn add(self, other: char) -> String {
        self + String::from_char(other)
    }
}

impl Add<usize> for String {
    type Output = String;
    fn add(self, other: usize) -> String {
        self + String::from_num(other)
    }
}
