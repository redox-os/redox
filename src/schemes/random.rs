use common::random::*;
use common::resource::*;
use common::string::*;

use alloc::boxed::*;

use programs::session::*;

pub struct RandomScheme;

impl SessionModule for RandomScheme {
    fn scheme(&self) -> String {
        return "random".to_string();
    }

    fn open(&mut self, url: &URL) -> Box<Resource> {
        return box VecResource::new(String::from_num(rand()).to_utf8());
    }
}
