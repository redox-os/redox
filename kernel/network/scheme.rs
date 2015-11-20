use alloc::boxed::Box;

use collections::vec::Vec;
use collections::vec_deque::VecDeque;

use core::ops::DerefMut;

use scheduler::context::context_switch;
use common::debug;
use scheduler;

use schemes::{Resource, ResourceSeek, Url};

pub trait NetworkScheme {
    fn add(&mut self, resource: *mut NetworkResource);
    fn remove(&mut self, resource: *mut NetworkResource);
    fn sync(&mut self);
}

pub struct NetworkResource {
    pub nic: *mut NetworkScheme,
    pub ptr: *mut NetworkResource,
    pub inbound: VecDeque<Vec<u8>>,
    pub outbound: VecDeque<Vec<u8>>,
}

impl NetworkResource {
    pub fn new(nic: *mut NetworkScheme) -> Box<Self> {
        let mut ret = box NetworkResource {
            nic: nic,
            ptr: 0 as *mut NetworkResource,
            inbound: VecDeque::new(),
            outbound: VecDeque::new(),
        };

        unsafe {
            ret.ptr = ret.deref_mut();

            (*ret.nic).add(ret.ptr);
        }

        ret
    }
}

impl Resource for NetworkResource {
    fn dup(&self) -> Option<Box<Resource>> {
        let mut ret = box NetworkResource {
            nic: self.nic,
            ptr: 0 as *mut NetworkResource,
            inbound: self.inbound.clone(),
            outbound: self.outbound.clone(),
        };

        unsafe {
            ret.ptr = ret.deref_mut();

            (*ret.nic).add(ret.ptr);
        }

        Some(ret)
    }

    fn url(&self) -> Url {
        Url::from_str("network:")
    }

    fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        debug::d("TODO: Implement read for RTL8139\n");
        None
    }

    fn read_to_end(&mut self, vec: &mut Vec<u8>) -> Option<usize> {
        loop {
            unsafe {
                (*self.nic).sync();

                let reenable = scheduler::start_no_ints();
                let option = (*self.ptr).inbound.pop_front();
                scheduler::end_no_ints(reenable);

                if let Some(bytes) = option {
                    vec.push_all(&bytes);
                    return Some(bytes.len());
                }

                context_switch(false);
            }
        }
    }

    fn write(&mut self, buf: &[u8]) -> Option<usize> {
        unsafe {
            let reenable = scheduler::start_no_ints();
            (*self.ptr).outbound.push_back(Vec::from(buf));
            scheduler::end_no_ints(reenable);

            (*self.nic).sync();
        }

        Some(buf.len())
    }

    fn seek(&mut self, _: ResourceSeek) -> Option<usize> {
        None
    }

    fn sync(&mut self) -> bool {
        false
    }
}

impl Drop for NetworkResource {
    fn drop(&mut self) {
        unsafe {
            (*self.nic).remove(self.ptr);
        }
    }
}
