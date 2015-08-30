use core::clone::Clone;
use core::cmp::PartialEq;
use core::iter::Iterator;
use core::mem::size_of;
use core::ops::Add;
use core::ops::Drop;
use core::ops::Index;
use core::option::Option;
use core::ptr;
use core::slice::SliceExt;
use core::str::StrExt;

use common::debug::*;
use common::memory::*;
use common::vec::*;

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
                if self.seperator == self.string.substr(i, self.seperator.len()){
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
    pub data: *const char,
    pub length: usize
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

        unsafe {
            let data = alloc(length * size_of::<char>()) as *mut char;

            let mut i = 0;
            for c in s.chars() {
                ptr::write(data.offset(i), c);
                i += 1;
            }

            String {
                data: data,
                length: length
            }
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

        unsafe {
            let data = alloc(length * size_of::<char>()) as *mut char;

            let mut i = 0;
            for c in s {
                if i >= length {
                    break;
                }
                ptr::write(data.offset(i as isize), ptr::read(c) as char);
                i += 1;
            }

            String {
                data: data,
                length: length
            }
        }
    }

    pub fn from_utf8(vec: &Vec<u8>) -> String {
        // TODO
        return String::from_c_slice(vec.as_slice());
    }

    pub unsafe fn from_c_str(s: *const u8) -> String {
        let mut length = 0;
        loop {
            if ptr::read(((s as usize) + length) as *const u8) == 0 {
                break;
            }
            length += 1;
        }

        if length == 0 {
            return String::new();
        }

        let data = alloc(length * size_of::<char>());

        for i in 0..length {
            ptr::write(((data + i * size_of::<char>()) as *mut char), ptr::read((((s as usize) + i) as *const u8)) as char);
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

        unsafe {
            let data = alloc(length * size_of::<char>()) as *mut char;

            let mut digit_num = num;
            for i in 0..length {
                let mut digit = (digit_num % radix) as u8;
                if digit > 9 {
                    digit += 'A' as u8 - 10;
                }else{
                    digit += '0' as u8;
                }

                ptr::write(data.offset((length - 1 - i) as isize), digit as char);
                digit_num /= radix;
            }

            String {
                data: data,
                length: length
            }
        }
    }

    pub fn from_num_radix_signed(num: isize, radix: usize) -> String {
        if num >= 0 {
            return String::from_num_radix(num as usize, radix);
        }else{
            return "-".to_string() + String::from_num_radix((-num) as usize, radix);
        }
    }

    pub fn from_char(c: char) -> String {
        if c == '\0' {
            return String::new();
        }

        unsafe{
            let data = alloc(size_of::<char>()) as *mut char;
            ptr::write(data, c);

            String {
                data: data,
                length: 1
            }
        }
    }

    pub fn from_num(num: usize) -> String {
        String::from_num_radix(num, 10)
    }

    pub fn from_num_signed(num: isize) -> String {
        String::from_num_radix_signed(num, 10)
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

        unsafe {
            let data = alloc(length * size_of::<char>()) as *mut char;

            for k in i..j {
                ptr::write(data.offset((k - i) as isize), ptr::read(self.data.offset(k as isize)));
            }

            String {
                data: data,
                length: length
            }
        }
    }

    pub fn find(&self, other: String) -> Option<usize> {
        if self.len() >= other.len() {
            for i in 0..self.len() + 1 - other.len() {
                if self.substr(i, other.len()) == other {
                    return Option::Some(i);
                }
            }
        }
        return Option::None;
    }

    pub fn starts_with(&self, other: String) -> bool {
        if self.len() >= other.len() {
            return self.substr(0, other.len()) == other;
        }else{
            return false;
        }
    }

    pub fn ends_with(&self, other: String) -> bool {
        if self.len() >= other.len() {
            return self.substr(self.len() - other.len(), other.len()) == other;
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

    pub fn to_utf8(&self) -> Vec<u8> {
        let mut vec: Vec<u8> = Vec::new();

        for c in self.chars() {
            let u = c as usize;
            if u < 0x80 {
                vec.push(u as u8);
            }else if u < 0x800 {
                vec.push(0b11000000 | ((u >> 6) as u8 & 0b00011111));
                vec.push(0b10000000 | (u as u8 & 0b00111111));
            }else if u < 0x10000 {
                vec.push(0b11100000 | ((u >> 12) as u8 & 0b00001111));
                vec.push(0b10000000 | ((u >> 6) as u8 & 0b00111111));
                vec.push(0b10000000 | (u as u8 & 0b00111111));
            }else{
                d("Unhandled to_utf8 code ");
                dh(u);
                dl();
            }
        }

        return vec;
    }

    pub unsafe fn to_c_str(&self) -> *const u8 {
        let length = self.len() + 1;

        let data = alloc(length);

        for i in 0..self.len() {
            ptr::write((data + i) as *mut u8, ptr::read(((self.data as usize) + i * size_of::<char>()) as *const char) as u8);
        }
        ptr::write((data + self.len()) as *mut u8, 0);

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

    pub fn to_num_radix_signed(&self, radix: usize) -> isize {
        if self[0] == '-' {
            return -(self.substr(1, self.len() - 1).to_num_radix(radix) as isize);
        }else{
            return self.to_num_radix(radix) as isize;
        }
    }

    pub fn to_num(&self) -> usize {
        self.to_num_radix(10)
    }

    pub fn to_num_signed(&self) -> isize {
        self.to_num_radix_signed(10)
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

impl PartialEq for String {
    fn eq(&self, other: &Self) -> bool{
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
}

impl Clone for String {
    fn clone(&self) -> Self{
        return self.substr(0, self.len());
    }
}

impl Drop for String {
    fn drop(&mut self){
        unsafe {
            unalloc(self.data as usize);
            self.data = 0 as *const char;
            self.length = 0;
        }
    }
}

impl Add for String {
    type Output = String;
    fn add(self, other: String) -> String {
        let length = self.length + other.length;

        if length == 0 {
            return String::new();
        }

        unsafe {
            let data = alloc(length * size_of::<char>()) as *mut char;

            let mut i = 0;
            for c in self.chars() {
                ptr::write(data.offset(i), c);
                i += 1;
            }
            for c in other.chars() {
                ptr::write(data.offset(i), c);
                i += 1;
            }

            String {
                data: data,
                length: length
            }
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

impl Add<isize> for String {
    type Output = String;
    fn add(self, other: isize) -> String {
        self + String::from_num_signed(other)
    }
}
