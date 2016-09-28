use alloc::arc::{Arc, Weak};
use collections::{BTreeMap, VecDeque};
use core::mem;
use core::sync::atomic::{AtomicUsize, Ordering};
use spin::{Mutex, RwLock};

use context;
use syscall::data::Event;
use syscall::error::*;
use syscall::scheme::Scheme;

pub struct EventScheme {
    next_id: AtomicUsize,
    handles: RwLock<BTreeMap<usize, Weak<Mutex<VecDeque<Event>>>>>
}

impl EventScheme {
    pub fn new() -> EventScheme {
        EventScheme {
            next_id: AtomicUsize::new(0),
            handles: RwLock::new(BTreeMap::new())
        }
    }
}

impl Scheme for EventScheme {
    fn open(&self, _path: &[u8], _flags: usize) -> Result<usize> {
        let handle = {
            let contexts = context::contexts();
            let context_lock = contexts.current().ok_or(Error::new(ESRCH))?;
            let context = context_lock.read();
            context.events.clone()
        };

        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        self.handles.write().insert(id, Arc::downgrade(&handle));

        Ok(id)
    }

    fn dup(&self, id: usize) -> Result<usize> {
        let handle = {
            let handles = self.handles.read();
            let handle_weak = handles.get(&id).ok_or(Error::new(EBADF))?;
            handle_weak.upgrade().ok_or(Error::new(EBADF))?
        };

        let new_id = self.next_id.fetch_add(1, Ordering::SeqCst);
        self.handles.write().insert(new_id, Arc::downgrade(&handle));
        Ok(new_id)
    }

    fn read(&self, id: usize, buf: &mut [u8]) -> Result<usize> {
        let handle = {
            let handles = self.handles.read();
            let handle_weak = handles.get(&id).ok_or(Error::new(EBADF))?;
            handle_weak.upgrade().ok_or(Error::new(EBADF))?
        };

        let event_size = mem::size_of::<Event>();
        let len = buf.len()/event_size;
        if len > 0 {
            loop {
                let mut i = 0;
                {
                    let mut events = handle.lock();
                    while ! events.is_empty() && i < len {
                        let event = events.pop_front().unwrap();
                        unsafe { *(buf.as_mut_ptr() as *mut Event).offset(i as isize) = event; }
                        i += 1;
                    }
                }

                if i > 0 {
                    return Ok(i * event_size);
                } else {
                    unsafe { context::switch(); } //TODO: Block
                }
            }
        } else {
            Ok(0)
        }
    }

    fn fsync(&self, id: usize) -> Result<usize> {
        let handles = self.handles.read();
        let handle_weak = handles.get(&id).ok_or(Error::new(EBADF))?;
        handle_weak.upgrade().ok_or(Error::new(EBADF)).and(Ok(0))
    }

    fn close(&self, id: usize) -> Result<usize> {
        self.handles.write().remove(&id).ok_or(Error::new(EBADF)).and(Ok(0))
    }
}
