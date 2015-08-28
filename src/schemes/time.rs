use programs::common::*;

pub struct TimeScheme;

impl SessionItem for TimeScheme {
    fn scheme(&self) -> String {
        return "time".to_string();
    }

    fn open(&mut self, url: &URL) -> Box<Resource> {
        let clock;
        unsafe{
            let reenable = start_no_ints();
            clock = ::clock_monotonic;
            end_no_ints(reenable);
        }

        return box VecResource::new(ResourceType::File, clock.to_string().to_utf8());
    }
}
