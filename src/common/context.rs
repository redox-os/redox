use alloc::boxed::*;

use core::ptr;

use common::debug::*;
use common::event::*;
use common::queue::*;
use common::memory::*;
use common::scheduler::*;
use common::vec::*;

use syscall::call::sys_exit;

pub const CONTEXT_STACK_SIZE: usize = 1024*1024;

pub static mut contexts_ptr: *mut Box<Vec<Context>> = 0 as *mut Box<Vec<Context>>;
pub static mut context_i: usize = 0;

pub unsafe extern "cdecl" fn context_box(box_fn_ptr: usize){
    let box_fn = ptr::read(box_fn_ptr as *mut Box<FnBox()>);
    unalloc(box_fn_ptr);
    box_fn();
}

pub unsafe extern "cdecl" fn context_exit() -> !{
    loop {
        sys_exit();
    }
}
/* TODO
pub unsafe fn context_event_wait() -> Event {
    loop{
        let mut event_option = Option::None;

        let reenable = start_no_ints();

        if contexts_ptr as usize > 0 {
            let contexts = &mut *(*contexts_ptr);

            match context.get(context_i) {
                Option::Some(current) => event_option = current.events.pop(),
                Option::None => ()
            }
        }

        end_no_ints(reenable);

        match event_option {
            Option::Some(event) => return event,
            Option::None => sys_yield()
        }
    }
}
*/
pub struct Context {
    pub stack: usize,
    pub stack_ptr: u32,
    pub fx: usize,
    pub events: Queue<Event>

}

impl Context {
    pub unsafe fn root() -> Context {
        let ret = Context {
            stack: 0,
            stack_ptr: 0,
            fx: alloc(512),
            events: Queue::new()
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
            fx: alloc(512),
            events: Queue::new()
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

    pub fn spawn(box_fn: Box<FnBox()>) {
        unsafe{
            let box_fn_ptr: *mut Box<FnBox()> = alloc_type();
            ptr::write(box_fn_ptr, box_fn);

            let mut context_box_args: Vec<usize> = Vec::new();
            context_box_args.push(box_fn_ptr as usize);

            let reenable = start_no_ints();
            if contexts_ptr as usize > 0 {
                (*contexts_ptr).push(Context::new(context_box as usize, &context_box_args));
            }
            end_no_ints(reenable);
        }
    }

    pub unsafe fn push(&mut self, data: u32){
        self.stack_ptr -= 4;
        *(self.stack_ptr as *mut u32) = data;
    }

    #[inline(never)]
    pub unsafe fn switch(&mut self, other: &mut Context){
        asm!("fxsave [edi]
            fxrstor [esi]"
            :
            : "{edi}"(self.fx), "{esi}"(other.fx)
            : "memory"
            : "intel", "volatile");
        asm!("pushfd
            pushad
            mov [edi], esp
            mov esp, [esi]
            popad
            popfd"
            :
            : "{edi}"(&mut self.stack_ptr), "{esi}"(&mut other.stack_ptr)
            : "memory"
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
