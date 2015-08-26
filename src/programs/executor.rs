use core::mem;
use core::sync::atomic::*;

use common::elf::*;

use programs::common::*;

pub struct Executor {
    executable: ELF,
    mapped: AtomicUsize,
    entry: usize,
    exit: usize,
    draw: usize,
    on_key: usize,
    on_mouse: usize
}

impl Executor {
    pub fn new() -> Executor {
        Executor {
            executable: ELF::new(),
            mapped: AtomicUsize::new(0),
            entry: 0,
            exit: 0,
            draw: 0,
            on_mouse: 0,
            on_key: 0
        }
    }

    unsafe fn entry(&mut self){
        if self.executable.can_call(self.entry){
            //Rediculous call mechanism
            self.unsafe_map();
            let fn_ptr: *const usize = &self.entry;
            (*(fn_ptr as *const fn()))();
            self.unsafe_unmap();
        }
    }

    unsafe fn unsafe_map(&mut self){
        let mapped = self.mapped.fetch_add(1, Ordering::SeqCst);
        if self.executable.data > 0 && mapped == 0{
            self.executable.map();
        }
    }

    unsafe fn unsafe_unmap(&mut self){
        let mapped = self.mapped.fetch_sub(1, Ordering::SeqCst);
        if self.executable.data > 0 && mapped == 1{
            self.executable.unmap();
        }
    }
}

impl Drop for Executor {
    fn drop(&mut self){
        unsafe{
            if self.executable.can_call(self.exit){
                //Rediculous call mechanism
                self.unsafe_map();
                let fn_ptr: *const usize = &self.exit;
                (*(fn_ptr as *const fn()))();
                self.unsafe_unmap();
            }
        }
    }
}

impl SessionItem for Executor {
    fn load(&mut self, url: &URL){
        let mut resource = url.open();

        let mut vec: Vec<u8> = Vec::new();
        match resource.read_to_end(&mut vec){
            Option::Some(_) => {
                unsafe{
                    self.executable = ELF::from_data(vec.as_ptr() as usize);
                    //self.executable.d();

                    self.entry = self.executable.entry();
                    self.exit = self.executable.symbol("exit".to_string());
                    self.draw = self.executable.symbol("draw".to_string());
                    self.on_key = self.executable.symbol("on_key".to_string());
                    self.on_mouse = self.executable.symbol("on_mouse".to_string());

                    self.entry();
                }
            },
            Option::None => ()
        }
    }

    fn draw(&self, display: &Display) -> bool{
        unsafe {
            if self.executable.can_call(self.draw){
                //Rediculous call mechanism
                let self_mut: *mut Executor = mem::transmute(self);

                (*self_mut).unsafe_map();
                let fn_ptr: *const usize = &self.draw;
                let ret = (*(fn_ptr as *const fn(&Display) -> bool))(display);
                (*self_mut).unsafe_unmap();

                return ret;
            }
        }
        return false;
    }

    fn on_key(&mut self, key_event: KeyEvent){
        unsafe {
            if self.executable.can_call(self.on_key){
                //Rediculous call mechanism
                self.unsafe_map();
                let fn_ptr: *const usize = &self.on_key;
                (*(fn_ptr as *const fn(KeyEvent)))(key_event);
                self.unsafe_unmap();
            }
        }
    }

    fn on_mouse(&mut self, mouse_event: MouseEvent, allow_catch: bool) -> bool{
        unsafe {
            if self.executable.can_call(self.on_mouse){
                //Rediculous call mechanism
                self.unsafe_map();
                let fn_ptr: *const usize = &self.on_mouse;
                let ret = (*(fn_ptr as *const fn(MouseEvent, bool) -> bool))(mouse_event, allow_catch);
                self.unsafe_unmap();
                return ret;
            }
        }
        return false;
    }
}
