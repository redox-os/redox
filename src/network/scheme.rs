use alloc::boxed::*;

use core::ops::DerefMut;

use common::debug::*;
use common::queue::*;
use common::resource::*;
use common::scheduler::*;
use common::string::*;
use common::vec::*;

use syscall::call::sys_yield;

pub trait NetworkScheme {
    fn add(&mut self, resource: *mut NetworkResource);
    fn remove(&mut self, resource: *mut NetworkResource);
    fn sync(&mut self);
}

pub struct NetworkResource {
    pub nic: *mut NetworkScheme,
    pub ptr: *mut NetworkResource,
    pub inbound: Queue<Vec<u8>>,
    pub outbound: Queue<Vec<u8>>
}

impl NetworkResource {
    pub fn new(nic: *mut NetworkScheme) -> Box<NetworkResource> {
        let mut ret = box NetworkResource {
            nic: nic,
            ptr: 0 as *mut NetworkResource,
            inbound: Queue::new(),
            outbound: Queue::new()
        };

        unsafe{
            ret.ptr = ret.deref_mut();

            (*ret.nic).add(ret.ptr);
        }

        return ret;
    }
}

impl Resource for NetworkResource {
    fn url(&self) -> URL {
        return URL::from_string(&"network://".to_string());
    }

    fn stat(&self) -> ResourceType {
        return ResourceType::File;
    }

    fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        d("TODO: Implement read for RTL8139\n");
        return Option::None;
    }

    fn read_to_end(&mut self, vec: &mut Vec<u8>) -> Option<usize> {
        loop {
            unsafe{
                (*self.nic).sync();

                let reenable = start_no_ints();
                let option = (*self.ptr).inbound.pop();
                end_no_ints(reenable);

                if let Option::Some(bytes) = option {
                    vec.push_all(&bytes);
                    return Option::Some(bytes.len());
                }

                sys_yield();
            }
        }
    }

    fn write(&mut self, buf: &[u8]) -> Option<usize> {
        unsafe{
            let reenable = start_no_ints();
            (*self.ptr).outbound.push(Vec::from_raw_buf(buf.as_ptr(), buf.len()));
            end_no_ints(reenable);

            (*self.nic).sync();
        }

        return Option::Some(buf.len());
    }

    fn seek(&mut self, pos: ResourceSeek) -> Option<usize> {
        return Option::None;
    }

    fn flush(&mut self) -> bool {
        return false;
    }
}

impl Drop for NetworkResource {
    fn drop(&mut self){
        unsafe {
            (*self.nic).remove(self.ptr);
        }
    }
}
