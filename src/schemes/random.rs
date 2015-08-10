use common::random::*;
use common::string::*;
use common::url::*;

use alloc::boxed::*;

use programs::session::*;

pub struct RandomScheme;

impl SessionModule for RandomScheme {
    fn scheme(&self) -> String {
        return "random".to_string();
    }

    #[allow(unused_variables)]
    fn on_url(&mut self, session: &Session, url: &URL, callback: Box<FnBox(String)>){
        callback(String::from_num(rand()));
    }
}
