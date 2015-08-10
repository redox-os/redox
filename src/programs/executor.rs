use common::debug::*;
use common::elf::*;
use common::string::*;

use drivers::keyboard::*;
use drivers::mouse::*;

use filesystems::unfs::*;

use programs::session::*;

pub struct Executor {
    executable: ELF,
    mapped: bool,
    entry: usize,
    draw: usize,
    on_key: usize,
    on_mouse: usize
}

impl Executor {
    unsafe fn entry(&mut self){
        if self.executable.can_call(self.entry){
            //Rediculous call mechanism
            self.unsafe_map();
            let fn_ptr: *const usize = &self.entry;
            (*(fn_ptr as *const fn()))();
            self.unsafe_unmap();
        }
    }
}

impl SessionItem for Executor {
    fn new() -> Executor {
        Executor {
            executable: ELF::new(),
            mapped: false,
            entry: 0,
            draw: 0,
            on_mouse: 0,
            on_key: 0
        }
    }

    #[allow(unused_variables)]
    fn load(&mut self, session: &Session, filename: String){
        if filename.len() > 0{
            unsafe{
                self.executable = ELF::from_data(UnFS::new().load(filename));
                //ret.executable.d();

                self.entry = self.executable.entry();
                self.draw = self.executable.symbol("draw".to_string());
                self.on_key = self.executable.symbol("on_key".to_string());
                self.on_mouse = self.executable.symbol("on_mouse".to_string());

                self.entry();
            }
        }
    }

    fn draw(&mut self, session: &Session, updates: &mut SessionUpdates) -> bool{
        unsafe {
            if self.executable.can_call(self.draw){
                //Rediculous call mechanism
                self.unsafe_map();
                let fn_ptr: *const usize = &self.draw;
                let ret = (*(fn_ptr as *const fn(&Session, &mut SessionUpdates) -> bool))(session, updates);
                self.unsafe_unmap();

                return ret;
            }
        }
        return false;
    }

    fn on_key(&mut self, session: &Session, updates: &mut SessionUpdates, key_event: KeyEvent){
        unsafe {
            if self.executable.can_call(self.on_key){
                //Rediculous call mechanism
                self.unsafe_map();
                let fn_ptr: *const usize = &self.on_key;
                (*(fn_ptr as *const fn(&Session, &mut SessionUpdates, KeyEvent)))(session, updates, key_event);
                self.unsafe_unmap();
            }
        }
    }

    fn on_mouse(&mut self, session: &Session, updates: &mut SessionUpdates, mouse_event: MouseEvent, allow_catch: bool) -> bool{
        unsafe {
            if self.executable.can_call(self.on_mouse){
                //Rediculous call mechanism
                self.unsafe_map();
                let fn_ptr: *const usize = &self.on_mouse;
                let ret = (*(fn_ptr as *const fn(&Session, &mut SessionUpdates, MouseEvent, bool) -> bool))(session, updates, mouse_event, allow_catch);
                self.unsafe_unmap();
                return ret;
            }
        }
        return false;
    }

    unsafe fn unsafe_map(&mut self){
        if self.executable.data > 0 && !self.mapped{
            self.executable.map();
            self.mapped = true;
        }
    }

    unsafe fn unsafe_unmap(&mut self){
        if self.executable.data > 0 && self.mapped {
            self.executable.unmap();
            self.mapped = false;
        }
    }
}
