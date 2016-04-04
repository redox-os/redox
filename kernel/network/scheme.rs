use alloc::boxed::Box;

use collections::vec::Vec;
use collections::vec_deque::VecDeque;

use core::ops::DerefMut;

use fs::Resource;

use system::error::Result;

use sync::{Intex, WaitQueue};

pub trait NetworkScheme {
    fn add(&mut self, resource: *mut NetworkResource);
    fn remove(&mut self, resource: *mut NetworkResource);
    fn sync(&mut self);
}

pub struct NetworkResource {
    pub nic: *mut NetworkScheme,
    pub ptr: *mut NetworkResource,
    pub inbound: WaitQueue<Vec<u8>>,
    pub outbound: Intex<VecDeque<Vec<u8>>>,
}

impl NetworkResource {
    pub fn new(nic: *mut NetworkScheme) -> Box<Self> {
        let mut ret = box NetworkResource {
            nic: nic,
            ptr: 0 as *mut NetworkResource,
            inbound: WaitQueue::new(),
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
    fn dup(&self) -> Result<Box<Resource>> {
        let mut ret = box NetworkResource {
            nic: self.nic,
            ptr: 0 as *mut NetworkResource,
            inbound: self.inbound.clone(),
            outbound: Intex::new(self.outbound.lock().clone()),
        };

        unsafe {
            ret.ptr = ret.deref_mut();

            (*ret.nic).add(ret.ptr);
        }

        Ok(ret)
    }

    fn path(&self, buf: &mut [u8]) -> Result<usize> {
        let path = b"network:";

        let mut i = 0;
        while i < buf.len() && i < path.len() {
            buf[i] = path[i];
            i += 1;
        }

        Ok(i)
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let bytes = unsafe {
            (*self.nic).sync();
            (*self.ptr).inbound.receive()
        };

        let mut i = 0;
        while i < bytes.len() && i < buf.len() {
            buf[i] = bytes[i];
            i += 1;
        }

        return Ok(bytes.len());
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        unsafe {
            (*self.ptr).outbound.lock().push_back(Vec::from(buf));

            (*self.nic).sync();
        }

        Ok(buf.len())
    }

    fn sync(&mut self) -> Result<()> {
        unsafe {
            (*self.nic).sync();
        }
        Ok(())
    }
}

impl Drop for NetworkResource {
    fn drop(&mut self) {
        unsafe {
            (*self.nic).remove(self.ptr);
        }
    }
}
