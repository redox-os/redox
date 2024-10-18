pub use pbr::ProgressBar;

use std::io::{Read, Result, Write};

pub struct ProgressBarRead<'p, 'r, P: Write + 'p, R: Read + 'r> {
    pb: &'p mut ProgressBar<P>,
    r: &'r mut R,
}

impl<'p, 'r, P: Write, R: Read> ProgressBarRead<'p, 'r, P, R> {
    pub fn new(pb: &'p mut ProgressBar<P>, r: &'r mut R) -> ProgressBarRead<'p, 'r, P, R> {
        ProgressBarRead { pb, r }
    }
}

impl<'p, 'r, P: Write, R: Read> Read for ProgressBarRead<'p, 'r, P, R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let count = self.r.read(buf)?;
        self.pb.add(count as u64);
        Ok(count)
    }
}
