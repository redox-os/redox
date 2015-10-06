use collections::string::*;
use collections::vec::Vec;

pub struct WAV {
    pub channels: u16,
    pub sample_rate: u32,
    pub sample_bits: u16,
    pub data: Vec<u8>,
}

impl WAV {
    pub fn new() -> Self {
        WAV {
            channels: 0,
            sample_rate: 0,
            sample_bits: 0,
            data: Vec::new(),
        }
    }

    pub fn from_data(file_data: &Vec<u8>) -> Self {
        let mut ret = WAV::new();

        let get = |i: usize| -> u8 {
            match file_data.get(i) {
                Option::Some(byte) => *byte,
                Option::None => 0,
            }
        };

        let getw = |i: usize| -> u16 { (get(i) as u16) + ((get(i + 1) as u16) << 8) };

        let getd = |i: usize| -> u32 {
            (get(i) as u32) + ((get(i + 1) as u32) << 8) + ((get(i + 2) as u32) << 16) +
            ((get(i + 3) as u32) << 24)
        };

        let gets = |start: usize, len: usize| -> String {
            let mut ret = String::new();
            for i in start..start + len {
                ret = ret + &(get(i) as char).to_string();
            }
            ret
        };

        let mut i = 0;
        let root_type = gets(i, 4);
        i += 4;
        //let root_size = getd(i);
        i += 4;

        if root_type == "RIFF".to_string() {
            let media_type = gets(i, 4);
            i += 4;

            if media_type == "WAVE".to_string() {
                loop {
                    let chunk_type = gets(i, 4);
                    i += 4;
                    let chunk_size = getd(i);
                    i += 4;

                    if chunk_type.len() == 0 || chunk_size == 0 {
                        break;
                    }

                    if chunk_type == "fmt ".to_string() {
                        ret.channels = getw(i + 2);
                        ret.sample_rate = getd(i + 4);
                        ret.sample_bits = getw(i + 0xE);
                    }

                    if chunk_type == "data".to_string() {
                        ret.data = file_data[i .. chunk_size as usize].to_vec();
                    }

                    i += chunk_size as usize;
                }
            }
        }

        ret
    }
}
