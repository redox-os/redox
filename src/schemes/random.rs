use common::random::*;
use common::string::*;
use common::url::*;

use programs::session::*;

pub struct RandomScheme;

impl SessionScheme for RandomScheme {
    fn scheme(&self) -> String {
        return "random".to_string();
    }

    #[allow(unused_variables)]
    fn on_url(&mut self, session: &Session, url: &URL) -> String {
        return String::from_num(rand());
    }
}
