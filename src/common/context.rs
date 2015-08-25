use core::atomic::*;

use common::debug::*;
use common::memory::*;

pub const CONTEXT_STACK_SIZE: usize = 1024*1024;

pub struct Context {
    pub stack: usize,
    pub stack_ptr: u32,
    pub block: AtomicUsize
}

impl Context{
    pub unsafe fn root() -> Context {
        Context {
            stack: 0,
            stack_ptr: 0,
            block: AtomicUsize::new(0)
        }
    }

    pub unsafe fn new(callback: unsafe extern fn() -> !) -> Context {
        let stack = alloc(CONTEXT_STACK_SIZE);

        let mut ret = Context {
            stack: stack,
            stack_ptr: stack as u32,
            block: AtomicUsize::new(0)
        };

        ret.push(callback as u32); //EIP

        ret.push(0x1D1D1D1D); //EDI is a param
        ret.push(0x15151515); //ESI is a param

        ret.push(1 << 9); //Flags

        ret.push(0xAAAAAAAA); //EAX
        ret.push(0xCCCCCCCC); //ECX
        ret.push(0xDDDDDDDD); //EDX
        ret.push(0xBBBBBBBB); //EBX
        ret.push(0x59595959); //ESP (ignored)
        ret.push(0xB9B9B9B9); //EBP
        ret.push(0x51515151); //ESI
        ret.push(0xD1D1D1D1); //EDI

        return ret;
    }

    pub unsafe fn push(&mut self, data: u32){
        self.stack_ptr -= 4;
        *(self.stack_ptr as *mut u32) = data;
        d("Push ");
        dh(self.stack_ptr as usize);
        d(" ");
        dh(data as usize);
        dl();
    }

    #[inline(never)]
    pub unsafe fn swap(&mut self, other: &mut Context){
        asm!("pushfd
            pushad
            mov [edi], esp
            mov esp, [esi]
            popad
            popfd"
            :
            : "{edi}"(&mut self.stack_ptr), "{esi}"(&mut other.stack_ptr)
            :
            : "intel", "volatile");
    }
}

impl Drop for Context {
    fn drop(&mut self){
        if self.stack > 0 {
            unsafe {
                unalloc(self.stack);
            }
        }
    }
}
