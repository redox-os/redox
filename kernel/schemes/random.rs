use alloc::boxed::Box;

use common::random;

use schemes::{KScheme, Resource, Url, VecResource};

/// A pseudorandomness scheme
pub struct RandomScheme;

impl KScheme for RandomScheme {
    fn scheme(&self) -> &str {
        "random"
    }

    fn open(&mut self, _: &Url, _: usize) -> Option<Box<Resource>> {
        Some(box VecResource::new(Url::from_str("random://"), format!("{}", random::rand()).into_bytes()))
    }
}
