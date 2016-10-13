#![feature(question_mark)]

extern crate syscall;

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Read, Error, Result};
use std::os::unix::io::RawFd;

pub struct EventQueue<R> {
    /// The file to read events from
    file: File,
    /// A map of registered file descriptors to their handler callbacks
    callbacks: BTreeMap<RawFd, Box<FnMut(usize) -> Result<Option<R>>>>
}

impl<R> EventQueue<R> {
    /// Create a new event queue
    pub fn new() -> Result<EventQueue<R>> {
        Ok(EventQueue {
            file: File::open("event:")?,
            callbacks: BTreeMap::new()
        })
    }

    /// Add a file to the event queue, calling a callback when an event occurs
    ///
    /// The callback is given a mutable reference to the file and the event data
    /// (typically the length of data available for read)
    ///
    /// The callback returns Ok(None) if it wishes to continue the event loop,
    /// or Ok(Some(R)) to break the event loop and return the value.
    /// Err can be used to allow the callback to return an I/O error, and break the
    /// event loop
    pub fn add<F: FnMut(usize) -> Result<Option<R>> + 'static>(&mut self, fd: RawFd, callback: F) -> Result<()> {
        syscall::fevent(fd, syscall::EVENT_READ).map_err(|x| Error::from_sys(x))?;

        self.callbacks.insert(fd, Box::new(callback));

        Ok(())
    }

    /// Process the event queue until a callback returns Some(R)
    pub fn run(&mut self) -> Result<R> {
        loop {
            let mut event = syscall::Event::default();
            self.file.read(&mut event)?;
            if let Some(callback) = self.callbacks.get_mut(&event.id) {
                if let Some(ret) = callback(event.data)? {
                    return Ok(ret);
                }
            }
        }
    }
}
