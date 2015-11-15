use redox::hash::Hasher;

pub struct Djb2 {
    state: u64,
}

impl Djb2 {
    /// Create new DJB2 hasher
    pub fn new() -> Self {
        Djb2 {
            state: 5381,
        }
    }
}

impl Hasher for Djb2 {
    fn finish(&self) -> u64 {
        self.state
    }

    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.state = ((self.state << 5) + self.state) + b as u64;
        }
    }
}
