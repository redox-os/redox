use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};

fn main() {
   let mut input = File::open("unifont.hex").unwrap();
   let mut output = File::create("unifont.font").unwrap();
   let mut count = 0;
   for line_res in BufReader::new(input).lines() {
       let line = line_res.unwrap();

       let mut parts = line.split(":");
       let num = u32::from_str_radix(parts.next().unwrap(), 16).unwrap();
       assert_eq!(num, count);

       let mut data = [0; 16];
       let data_part = parts.next().unwrap();
       for i in 0..data.len() {
           data[i] = u8::from_str_radix(&data_part[i * 2 .. i * 2 + 2], 16).unwrap();
       }
       println!("{:>04X}:{:?}", num, data);

       output.write(&data).unwrap();

       count += 1;
   }
}
