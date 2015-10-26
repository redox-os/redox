use alloc::boxed::Box;

use common::context;
use common::scheduler;
use common::string::{String, ToString};

use schemes::{KScheme, Resource, URL, VecResource};

pub struct ContextScheme;

impl KScheme for ContextScheme {
    fn scheme(&self) -> String {
        "context".to_string()
    }

    fn open(&mut self, url: &URL) -> Option<Box<Resource>> {
        let i;
        let len;
        unsafe {
            let reenable = scheduler::start_no_ints();
            i = context::context_i;
            len = (*context::contexts_ptr).len();
            scheduler::end_no_ints(reenable);
        }

        Some(box VecResource::new(URL::from_str("context://"), ("Current: ".to_string() + i + "\nTotal: " + len).to_utf8()))
    }
}
