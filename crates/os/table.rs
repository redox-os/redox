use redox::prelude::v1::*;
use archive::Archive;

pub struct NodeTable<'a> {
    pub table: &'a [u8],
}

impl NodeTable {
    pub fn new(&self, b: &[u8]) -> Self {
        NodeTable {
            table: b,
        }
    }

    pub fn get_entry(&self, n: usize) -> Option<DataPtr> {
        DataPtr::from_bytes(n, &self.table[n * 16..n * 16 + 16])
    }

    pub fn get<T: Hash>(&self, key: T, probe: &mut usize) -> Option<DataPtr> {
        let hasher = Djb2::new();
        key.hash(hasher);

        let res = self.get_entry(
            hasher.finish() + *probe
        );

        *probe = match *probe + 1 {
            n if n < self.table.len() => n,
            n if n == self.table.len() => 0,
            // Just in case:
            n => n % self.table.len(),
        };

        res
    }

}
