use core::ptr;

use common::debug::*;
use common::memory::*;
use common::scheduler::*;
use common::vec::*;

pub const CONTEXT_STACK_SIZE: usize = 1024*1024;

pub unsafe extern "cdecl" fn context_exit() -> ! {
    loop{
        sched_exit();
    }
}

pub struct Context {
    pub stack: usize,
    pub stack_ptr: u32,
    pub fx: usize
}

impl Context {
    pub unsafe fn root() -> Context {
        let ret = Context {
            stack: 0,
            stack_ptr: 0,
            fx: alloc(512)
        };

        for i in 0..512 {
            ptr::write((ret.fx + i) as *mut u8, 0);
        }

        return ret;
    }

    pub unsafe fn new(call: usize, args: &Vec<usize>) -> Context {
        let stack = alloc(CONTEXT_STACK_SIZE);

        let mut ret = Context {
            stack: stack,
            stack_ptr: (stack + CONTEXT_STACK_SIZE) as u32,
            fx: alloc(512)
        };

        let ebp = ret.stack_ptr;

        for arg in args.iter() {
            ret.push(*arg as u32);
        }

        ret.push(context_exit as u32); //If the function call returns, we will exit
        ret.push(call as u32); //We will ret into this function call

        ret.push(0); //EDI is a param
        ret.push(0); //ESI is a param

        ret.push(1 << 9); //Flags

        let esp = ret.stack_ptr;

        ret.push(0); //EAX
        ret.push(0); //ECX
        ret.push(0); //EDX
        ret.push(0); //EBX
        ret.push(esp); //ESP (ignored)
        ret.push(ebp); //EBP
        ret.push(0); //ESI
        ret.push(0); //EDI

        for i in 0..512 {
            ptr::write((ret.fx + i) as *mut u8, 0);
        }

        return ret;
    }

    pub unsafe fn push(&mut self, data: u32){
        self.stack_ptr -= 4;
        *(self.stack_ptr as *mut u32) = data;
    }

    #[inline(never)]
    pub unsafe fn swap(&mut self, other: &mut Context){
        asm!("fxsave [edi]
            fxrstor [esi]"
            :
            : "{edi}"(self.fx), "{esi}"(other.fx)
            :
            : "intel", "volatile");
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

        if self.fx > 0 {
            unsafe {
                unalloc(self.fx);
            }
        }
    }
}
