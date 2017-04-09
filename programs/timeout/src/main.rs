extern crate event;
extern crate syscall;

use event::EventQueue;
use std::fs::File;
use std::io::{Result, Read, Write};
use std::mem;
use std::os::unix::io::AsRawFd;
use syscall::data::TimeSpec;
use syscall::flag::CLOCK_MONOTONIC;

fn main() {
    let mut event_queue = EventQueue::<TimeSpec>::new().expect("timeout: failed to create event queue");

    let path = format!("time:{}", CLOCK_MONOTONIC);

    let mut file = File::open(&path).expect(&format!("timeout: failed to open {}", path));

    let mut timeout = TimeSpec::default();
    file.read(&mut timeout).unwrap();
    println!("Current: {:?}", timeout);

    timeout.tv_sec += 1;
    println!("Setting timeout: {:?}", timeout);
    file.write(&timeout).unwrap();

    event_queue.add(file.as_raw_fd(), move |_count: usize| -> Result<Option<TimeSpec>> {
        let mut time = TimeSpec::default();
        if file.read(&mut time)? >= mem::size_of::<TimeSpec>() {
            if time.tv_sec > timeout.tv_sec
            || (time.tv_sec == timeout.tv_sec && time.tv_nsec >= timeout.tv_nsec )
            {
                return Ok(Some(time))
            }
        }
        Ok(None)
    }).expect("timeout: failed to poll time");

    event_queue.trigger_all(0).expect("timeout: failed to trigger events");

    let time = event_queue.run().expect("timeout: failed to run event loop");

    println!("Time passed: {:?}", time);
}
