use core::fmt;

pub struct Log {
    pub data: [u8; 65536],
    pub start: usize,
    pub end: usize
}

impl Log {
    pub fn new() -> Log {
        Log {
            data: [0; 65536],
            start: 0,
            end: 0
        }
    }

    fn move_start(&mut self) {
        self.start += 1;
        while self.start >= self.data.len() {
            self.start -= self.data.len();
        }
    }

    fn move_end(&mut self) {
        self.end += 1;
        while self.end >= self.data.len() {
            self.end -= self.data.len();
        }
        if self.end == self.start {
            'adjusting: loop {
                self.move_start();
                if self.data[self.start] == b'\n' || self.start == self.end {
                    self.move_start();
                    break 'adjusting;
                }
            }
        }
    }

    pub fn read_at(&self, pos: usize, buf: &mut [u8]) -> usize {
        let mut count = 0;
        let mut i = self.start + pos;
        while i >= self.data.len() {
            i -= self.data.len()
        }
        for byte in buf.iter_mut() {
            if i == self.end {
                break;
            }
            *byte = self.data[i];
            i += 1;
            while i >= self.data.len() {
                i -= self.data.len()
            }
            count += 1;
        }
        count
    }

    pub fn write(&mut self, buf: &[u8]) -> usize {
        let mut count = 0;
        for byte in buf.iter() {
            self.data[self.end] = *byte;
            self.move_end();
            count += 1;
        }
        count
    }
}

impl fmt::Write for Log {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        self.write(s.as_bytes());

        Ok(())
    }
}
