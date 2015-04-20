use core::iter::Iterator;
use core::ops::Add;
use core::ops::Drop;
use core::slice;
use core::str::StrExt;

use common::debug::*;
use common::memory::*;

pub struct String {
    data: *const char,
    length: u32
}

impl String {
    // TODO FromStr trait
    pub fn from_str(s: &str) -> String {
        let length = s.chars().count() as u32;
        let data = alloc(length * 4);
    
        let mut i = 0;
        for c in s.chars() {
            unsafe {
                *((data + i*4) as *mut char) = c;
            }
            i += 1;
        }
        
        d("Create ");
        dh(data);
        dl();
    
        let ret = String {
            data: data as *const char,
            length: length
        };
        
        ret
    }
    
    pub fn len(&self) -> u32 {
        self.length
    }
    
    // TODO: Str trait
    pub fn as_slice(&self) -> &[char] {
        unsafe {
            return slice::from_raw_parts(self.data, self.length as usize);
        }
    }
    
    pub fn d(&self){
        for character in self.as_slice() {
            dc(*character);
        }
    }
}

impl Drop for String {
    fn drop(&mut self){
        d("Drop ");
        dh(self.data as u32);
        dl();
        
        unalloc(self.data as u32);
    }
}

impl Add for String {
    type Output = String;
    fn add(self, other: String) -> String {
        let length = self.length + other.length;
        let data = alloc(length * 4);
    
        let mut i = 0;
        for c in self.as_slice() {
            unsafe {
                *((data + i*4) as *mut char) = *c;
            }
            i += 1;
        }
        for c in other.as_slice() {
            unsafe {
                *((data + i*4) as *mut char) = *c;
            }
            i += 1;
        }
        
        
        d("Create ");
        dh(data);
        dl();
    
        let ret = String {
            data: data as *const char,
            length: length
        };
        
        ret
    }
}