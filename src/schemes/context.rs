use programs::common::*;

use common::context::*;
use common::scheduler::*;

pub struct ContextScheme;

impl SessionItem for ContextScheme {
    fn scheme(&self) -> String {
        return "context".to_string();
    }

    fn open(&mut self, url: &URL) -> Box<Resource> {
        let i;
        let len;
        unsafe{
            let reenable = start_no_ints();
            i = context_i;
            len = (*contexts_ptr).len();
            end_no_ints(reenable);
        }

        return box VecResource::new(URL::from_string(&"context://".to_string()), ResourceType::File, ("Current: ".to_string() + i + "\nTotal: " + len).to_utf8());
    }
}
