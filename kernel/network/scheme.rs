use alloc::boxed::Box;

use arch::context::context_switch;

use collections::vec::Vec;
use collections::vec_deque::VecDeque;

use core::ops::DerefMut;

use fs::Resource;

use syscall::Result;

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
    fn dup(&self) -> Result<Box<Resource>> {
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
        loop {
            unsafe {
                {
                    (*self.nic).sync();

                    let option = (*self.ptr).inbound.lock().pop_front();

                    if let Some(bytes) = option {
                        let mut i = 0;
                        while i < bytes.len() && i < buf.len() {
                            buf[i] = bytes[i];
                            i += 1;
                        }
                        return Ok(bytes.len());
                    }
                }

                context_switch();
            }
        }
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
