use common::pio::*;

use core::str::StrExt;

pub unsafe fn serial_init(){
    outb(0x3F8 + 1, 0x00);
    outb(0x3F8 + 3, 0x80);
    outb(0x3F8 + 0, 0x03);
    outb(0x3F8 + 1, 0x00);
    outb(0x3F8 + 3, 0x03);
    outb(0x3F8 + 2, 0xC7);
    outb(0x3F8 + 4, 0x0B);
}

pub fn db(byte: u8){
    unsafe{
        outb(0x3F8, byte);
    }
}

pub fn dbh(byte: u8){
    let mut high = byte / 16;
    if high <= 9 {
        high += '0' as u8;
    }else{
        high -= 10;
        high += 'A' as u8;
    }
    db(high);

    let mut low = byte % 16;
    if low <= 9 {
        low += '0' as u8;
    }else{
        low -= 10;
        low += 'A' as u8;
    }
    db(low);
}

pub fn dh(num: usize){
    if num >= 256 {
        dh(num / 256);
    }
    dbh((num % 256) as u8);
}

pub fn dd(num: usize){
    if num >= 10 {
        dd(num / 10);
    }
    db('0' as u8 + (num % 10) as u8);
}

pub fn dc(character: char){
    db(character as u8);
}

pub fn dl(){
    dc('\n');
}

pub fn d(text: &str){
    for character in text.chars() {
        dc(character);
    }
}