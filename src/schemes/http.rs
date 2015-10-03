use alloc::boxed::Box;

use common::resource::{Resource, URL};
use common::string::{String, ToString};

use programs::common::SessionItem;

pub struct HTTPScheme;

impl SessionItem for HTTPScheme {
    fn scheme(&self) -> String {
        return "http".to_string();
    }

    fn open(&mut self, url: &URL) -> Box<Resource> {
        //TODO: DNS
        return URL::from_string(&("tcp://".to_string() + url.host() + ":80")).open();
    }
}
