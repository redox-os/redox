use alloc::boxed::Box;

use common::resource::{Resource, ResourceType, URL, VecResource};
use common::string::{String, ToString};

use common::scheduler::*;

use programs::common::SessionItem;

pub struct TimeScheme;

impl SessionItem for TimeScheme {
    fn scheme(&self) -> String {
        return "time".to_string();
    }

    fn open(&mut self, url: &URL) -> Box<Resource> {
        let clock_realtime;
        let clock_monotonic;
        unsafe {
            let reenable = start_no_ints();
            clock_realtime = ::clock_realtime;
            clock_monotonic = ::clock_monotonic;
            end_no_ints(reenable);
        }

        return box VecResource::new(URL::from_str("time://"),
                                    ResourceType::File,
                                    ("Time: ".to_string() + clock_realtime.to_string() +
                                     "\nUptime: " +
                                     clock_monotonic.to_string())
                                        .to_utf8());
    }
}
