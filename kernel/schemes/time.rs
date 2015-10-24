use alloc::boxed::Box;

use common::string::{String, ToString};
use common::scheduler;

use schemes::{KScheme, Resource, URL, VecResource};

/// A time scheme
pub struct TimeScheme;

impl KScheme for TimeScheme {
    fn scheme(&self) -> String {
        "time".to_string()
    }

    fn open(&mut self, url: &URL) -> Option<Box<Resource>> {
        let clock_realtime;
        let clock_monotonic;
        unsafe {
            let reenable = scheduler::start_no_ints();
            clock_realtime = ::clock_realtime;
            clock_monotonic = ::clock_monotonic;
            scheduler::end_no_ints(reenable);
        }

        let string = "Time: ".to_string() +
                    String::from_num_signed(clock_realtime.secs as isize) +
                    "\nUptime: " +
                    String::from_num_signed(clock_monotonic.secs as isize);

        Some(box VecResource::new(URL::from_str("time://"), string.to_utf8()))
    }
}
