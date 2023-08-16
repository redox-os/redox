pub use pbr::ProgressBar;

use std::io::{Read, Write, Result};

pub struct ProgressBarRead<'p, 'r, P: Write + 'p, R: Read + 'r> {
    pb: &'p mut ProgressBar<P>,
    r: &'r mut R,
}

impl<'p, 'r, P: Write, R: Read> ProgressBarRead<'p, 'r, P, R> {
    pub fn new(pb: &'p mut ProgressBar<P>, r: &'r mut R) -> ProgressBarRead<'p, 'r, P, R> {
        ProgressBarRead {
            pb,
            r
        }
    }
}

impl<'p, 'r, P: Write, R: Read> Read for ProgressBarRead<'p, 'r, P, R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let count = self.r.read(buf)?;
        self.pb.add(count as u64);
        Ok(count)
    }
}

pub struct ProgressBarWrite<'p, 'w, P: Write + 'p, W: Write + 'w> {
    pb: &'p mut ProgressBar<P>,
    w: &'w mut W,
}

impl<'p, 'w, P: Write, W: Write> ProgressBarWrite<'p, 'w, P, W> {
    pub fn _new(pb: &'p mut ProgressBar<P>, w: &'w mut W) -> ProgressBarWrite<'p, 'w, P, W> {
        ProgressBarWrite {
            pb,
            w
        }
    }
}

impl<'p, 'w, P: Write, W: Write> Write for ProgressBarWrite<'p, 'w, P, W> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let count = self.w.write(buf)?;
        self.pb.add(count as u64);
        Ok(count)
    }

    fn flush(&mut self) -> Result<()> {
        self.w.flush()
    }
}
