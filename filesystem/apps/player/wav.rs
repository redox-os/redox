/// A WAV file
// TODO: Follow naming conventions
pub struct WavFile {
    /// The number of channels
    pub channels: u16,
    /// The sample rate
    pub sample_rate: u32,
    /// The sample bits
    pub sample_bits: u16,
    /// The data
    pub data: Vec<u8>,
}

impl WavFile {
    /// Create a new empty WAV file
    pub fn new() -> Self {
        WavFile {
            channels: 0,
            sample_rate: 0,
            sample_bits: 0,
            data: Vec::new(),
        }
    }

    /// Create a WAV file from data
    pub fn from_data(file_data: &[u8]) -> Self {
        let mut ret = WavFile::new();

        let get = |i: usize| -> u8 {
            match file_data.get(i) {
                Some(byte) => *byte,
                None => 0,
            }
        };

        let getw = |i: usize| -> u16 { (get(i) as u16) + ((get(i + 1) as u16) << 8) };

        let getd = |i: usize| -> u32 {
            (get(i) as u32) + ((get(i + 1) as u32) << 8) + ((get(i + 2) as u32) << 16) +
            ((get(i + 3) as u32) << 24)
        };

        let gets = |start: usize, len: usize| -> String {
            (start..start + len).map(|i| get(i) as char).collect::<String>()
        };

        let mut i = 0;
        let root_type = gets(i, 4);
        i += 4;
        // let root_size = getd(i);
        i += 4;

        if root_type == "RIFF" {
            let media_type = gets(i, 4);
            i += 4;

            if media_type == "WAVE" {
                loop {
                    let chunk_type = gets(i, 4);
                    i += 4;
                    let chunk_size = getd(i);
                    i += 4;

                    if chunk_type.len() == 0 || chunk_size == 0 {
                        break;
                    }

                    if chunk_type == "fmt " {
                        ret.channels = getw(i + 2);
                        ret.sample_rate = getd(i + 4);
                        ret.sample_bits = getw(i + 0xE);
                    }

                    if chunk_type == "data" {
                        ret.data = file_data[i..chunk_size as usize].to_vec();
                    }

                    i += chunk_size as usize;
                }
            }
        }

        ret
    }
}
