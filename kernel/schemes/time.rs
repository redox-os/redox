use alloc::boxed::Box;

use scheduler;

use schemes::{KScheme, Resource, Url, VecResource};

/// A time scheme
pub struct TimeScheme;

impl KScheme for TimeScheme {
    fn scheme(&self) -> &str {
        "time"
    }

    fn open(&mut self, _: &Url, _: usize) -> Option<Box<Resource>> {
        let clock_realtime;
        let clock_monotonic;
        unsafe {
            let reenable = scheduler::start_no_ints();
            clock_realtime = ::clock_realtime;
            clock_monotonic = ::clock_monotonic;
            scheduler::end_no_ints(reenable);
        }

        let string = format!("Time: {}\nUptime: {}", clock_realtime.secs as isize, clock_monotonic.secs as isize);
        Some(box VecResource::new(Url::from_str("time://"), string.into_bytes()))
    }
}
