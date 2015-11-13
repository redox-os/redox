use redox::prelude::v1::*;
use archive::Archive;

pub struct NodeTable<'a> {
    pub offset: usize,
    pub tab: &'a [u8],
}

impl NodeTable {
    pub fn get_entry(&self, n: usize) -> Option<DataPtr> {
        DataPtr::from_bytes(n, self.offset + self.tab.len(), self.tab[n..n + 8])
    }

    pub fn get<T: Hash>(&self, key: T, probe: &mut usize) -> Option<DataPtr> {
        let hasher = Djb2::new();
        key.hash(hasher);

        self.get_entry(
            hasher.finish() + *probe
        )

        *probe = match *probe + 1 {
            n if n < self.tab.len() => n,
            n if n == self.tab.len() => 0,
            // Just in case:
            n => n % self.tab.len(),
        };
    }

    pub fn get_data<T: Hash>(&'a self, ar: &'a Archive, name: &'a [u8], timeout: u16) -> Option<Data<'a>> {
        let mut rnds = 0;
        let mut n = 0;
        let mut dat = self.get(key, &mut n);

        while dat.deref(ar).name() != name {
            dat = self.get(key, &mut n);
            rnds += 1;

            if rnds > timeout {
                return None;
            }
        }

        Some(dat)
    }


}
