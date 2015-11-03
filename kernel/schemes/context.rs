use alloc::boxed::Box;

use scheduler::context;
use scheduler;

use schemes::{KScheme, Resource, URL, VecResource};

pub struct ContextScheme;

impl KScheme for ContextScheme {
    fn scheme(&self) -> &str {
        "context"
    }

    fn open(&mut self, _: &URL) -> Option<Box<Resource>> {
        let i;
        let len;
        unsafe {
            let reenable = scheduler::start_no_ints();
            i = context::context_i;
            len = (*context::contexts_ptr).len();
            scheduler::end_no_ints(reenable);
        }

        Some(box VecResource::new(URL::from_str("context://"), format!("Current: {}\nTotal: {}", i, len).into_bytes()))
    }
}
