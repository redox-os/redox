use alloc::boxed::Box;

use collections::vec::Vec;
use collections::vec_deque::VecDeque;

use core::ops::DerefMut;

use scheduler::context::context_switch;
use common::debug;

use schemes::{Resource, ResourceSeek, Url};

use sync::Intex;

pub trait NetworkScheme {
    fn add(&mut self, resource: *mut NetworkResource);
    fn remove(&mut self, resource: *mut NetworkResource);
    fn sync(&mut self);
}

pub struct NetworkResource {
    pub nic: *mut NetworkScheme,
    pub ptr: *mut NetworkResource,
    pub inbound: Intex<VecDeque<Vec<u8>>>,
    pub outbound: Intex<VecDeque<Vec<u8>>>,
}

impl NetworkResource {
    pub fn new(nic: *mut NetworkScheme) -> Box<Self> {
        let mut ret = box NetworkResource {
            nic: nic,
            ptr: 0 as *mut NetworkResource,
            inbound: Intex::new(VecDeque::new()),
            outbound: Intex::new(VecDeque::new()),
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
            inbound: Intex::new(self.inbound.lock().clone()),
            outbound: Intex::new(self.outbound.lock().clone()),
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

    fn read(&mut self, _: &mut [u8]) -> Option<usize> {
        debug::d("TODO: Implement read for RTL8139\n");
        None
    }

    fn read_to_end(&mut self, vec: &mut Vec<u8>) -> Option<usize> {
        loop {
            unsafe {
                {
                    (*self.nic).sync();

                    let option = (*self.ptr).inbound.lock().pop_front();

                    if let Some(bytes) = option {
                        vec.push_all(&bytes);
                        return Some(bytes.len());
                    }
                }

                context_switch(false);
            }
        }
    }

    fn write(&mut self, buf: &[u8]) -> Option<usize> {
        unsafe {
            (*self.ptr).outbound.lock().push_back(Vec::from(buf));

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
