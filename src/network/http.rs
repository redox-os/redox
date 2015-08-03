use common::string::*;
use common::url::*;

use programs::session::*;

pub fn http_response(request: String, session: &Session) -> String {
    let mut path = "/".to_string();

    for row in request.split("\r\n".to_string()) {
        let mut i = 0;
        for col in row.split(" ".to_string()) {
            match i {
                1 => path = col,
                _ => ()
            }
            i += 1;
        }
        break;
    }

    return session.on_url(&URL::from_string("http://".to_string() + path));
}
