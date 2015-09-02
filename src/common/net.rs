use alloc::boxed::*;

use core::ops::DerefMut;
use core::result::Result;

use common::debug::*;
use common::queue::*;
use common::scheduler::*;
use common::string::*;
use common::vec::*;

use syscall::call::sys_tcp_listener_create;
use syscall::call::sys_tcp_listener_destroy;

#[derive(Copy, Clone)]
pub struct IPv4Addr {
    pub bytes: [u8; 4]
}

impl IPv4Addr {
    pub fn equals(&self, other: IPv4Addr) -> bool {
        for i in 0..4 {
            if self.bytes[i] != other.bytes[i] {
                return false;
            }
        }
        return true;
    }

    pub fn to_string(&self) -> String {
        let mut ret = String::new();

        for i in 0..4 {
            if i > 0 {
                ret = ret + '.';
            }
            ret = ret + self.bytes[i] as usize;
        }

        return ret;
    }

    pub fn d(&self){
        for i in 0..4 {
            if i > 0 {
                d(".");
            }
            dd(self.bytes[i] as usize);
        }
    }
}

#[derive(Copy, Clone)]
pub struct IPv6Addr {
    pub bytes: [u8; 16]
}

impl IPv6Addr {
    pub fn d(&self){
        for i in 0..16 {
            if i > 0 && i % 2 == 0 {
                d(":");
            }
            dbh(self.bytes[i]);
        }
    }
}

pub struct TcpStream{
    pub address: IPv4Addr,
    pub port: u16,
    pub data: Vec<u8>,
    pub response: Vec<u8>
}

pub struct TcpListener {
    pub port: u16,
    pub streams: Queue<Box<TcpStream>>,
    ptr: *mut TcpListener
}

impl TcpListener {
    pub fn bind(port: u16) -> Result<Box<TcpListener>, String> {
        let mut ret = box TcpListener {
            port: port,
            streams: Queue::new(),
            ptr: 0 as *mut TcpListener
        };

        ret.ptr = ret.deref_mut();

        if ret.ptr as usize > 0 {
            sys_tcp_listener_create(ret.ptr);
        }

        return Result::Ok(ret);
    }

    pub fn poll(&mut self) -> Option<Box<TcpStream>> {
        let stream_option;
        unsafe{
            let reenable = start_no_ints();
            stream_option = self.streams.pop();
            end_no_ints(reenable);
        }

        return stream_option;
    }
}

impl Drop for TcpListener {
    fn drop(&mut self) {
        if self.ptr as usize > 0{
            sys_tcp_listener_destroy(self.ptr);
        }
    }
}
