use alloc::boxed::Box;

use common::random;
use common::resource::{Resource, ResourceType, URL, VecResource};
use common::string::{String, ToString};

use programs::common::SessionItem;

pub struct RandomScheme;

impl SessionItem for RandomScheme {
    fn scheme(&self) -> String {
        return "random".to_string();
    }

    fn open(&mut self, url: &URL) -> Box<Resource> {
        return box VecResource::new(URL::from_str("random://"),
                                    ResourceType::File,
                                    String::from_num(random::rand()).to_utf8());
    }
}
