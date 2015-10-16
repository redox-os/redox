use redox::Box;
use redox::cell::UnsafeCell;
use redox::console::ConsoleWindow;
use redox::fs::file::File;
use redox::rc::Rc;
use redox::str;
use redox::string::*;
use redox::io::SeekFrom;

pub struct Scheme;

impl Scheme {
    fn scheme(&self) -> Box<Self> {
        box Scheme
    }

    fn open(&mut self, url: &URL) -> Option<Box<Resource>> {
        let i;
        let len;
        unsafe {
            /*
            let reenable = scheduler::start_no_ints();
            i = context_i;
            len = (*contexts_ptr).len();
            scheduler::end_no_ints(reenable);
            */
        }

        Some(box Resource::new(File::open("context://"), ("Current: ".to_string() + i + "\nTotal: " + len).to_utf8()))
    }
}
