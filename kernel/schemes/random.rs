use alloc::boxed::Box;

use common::random;

use schemes::{KScheme, Resource, URL, VecResource};

/// A pseudorandomness scheme
pub struct RandomScheme;

impl KScheme for RandomScheme {
    fn scheme(&self) -> &str {
        "random"
    }

    fn open(&mut self, _: &URL) -> Option<Box<Resource>> {
        Some(box VecResource::new(URL::from_str("random://"), format!("{}", random::rand()).into_bytes()))
    }
}
