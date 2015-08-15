use core::mem::size_of;
use core::ptr;

use alloc::boxed::*;

use common::memory::*;
use common::resource::*;
use common::string::*;
use common::url::*;

use programs::session::*;

pub fn open(url: &URL) -> Box<Resource> {
    unsafe{
        let url_ptr: *const URL = url;
        let resource_ptr: *mut Box<Resource> = alloc(size_of::<Box<Resource>>()) as *mut Box<Resource>;
        asm!("int 0x80"
            :
            : "{eax}"(1), "{ebx}"(url_ptr as u32), "{ecx}"(resource_ptr as u32)
            :
            : "intel");
        let resource = ptr::read(resource_ptr);
        unalloc(resource_ptr as usize);
        return resource;
    }
}

pub fn open_async(url: &URL, callback: Box<FnBox(Box<Resource>)>){
    unsafe{
        let url_ptr: *const URL = url;
        let callback_ptr: *mut Box<FnBox(Box<Resource>)> = alloc(size_of::<Box<FnBox(Box<Resource>)>>()) as *mut Box<FnBox(Box<Resource>)>;
        ptr::write(callback_ptr, callback);
        asm!("int 0x80"
            :
            : "{eax}"(2), "{ebx}"(url_ptr as u32), "{ecx}"(callback_ptr as u32)
            :
            : "intel");
    }
}
