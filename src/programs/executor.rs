use common::elf::*;
use common::string::*;

use drivers::keyboard::*;
use drivers::mouse::*;

use filesystems::unfs::*;

use programs::session::*;

pub struct Executor {
    executable: ELF,
    entry: usize,
    draw: usize,
    on_key: usize,
    on_mouse: usize
}

impl Executor {
    unsafe fn entry(&self){
        if self.executable.can_call(self.entry){
            //Rediculous call mechanism
            self.executable.map();
            let fn_ptr: *const usize = &self.entry;
            (*(fn_ptr as *const fn()))();
            self.executable.unmap();
        }
    }
}

impl SessionItem for Executor {
    fn new(file: String) -> Executor {
        let mut ret = Executor {
            executable: ELF::new(),
            entry: 0,
            draw: 0,
            on_mouse: 0,
            on_key: 0
        };

        if file.len() > 0{
            unsafe{
                ret.executable = ELF::from_data(UnFS::new().load(file));
                //ret.executable.d();

                ret.entry = ret.executable.entry();
                ret.draw = ret.executable.symbol("draw".to_string());
                ret.on_key = ret.executable.symbol("on_key".to_string());
                ret.on_mouse = ret.executable.symbol("on_mouse".to_string());

                ret.entry();
            }
        }

        return ret;
    }

    fn draw(&mut self, session: &Session, updates: &mut SessionUpdates) -> bool{
        unsafe {
            if self.executable.can_call(self.draw){
                //Rediculous call mechanism
                self.executable.map();
                let fn_ptr: *const usize = &self.draw;
                let ret = (*(fn_ptr as *const fn(&Session, &mut SessionUpdates) -> bool))(session, updates);
                self.executable.unmap();

                return ret;
            }
        }
        return false;
    }

    fn on_key(&mut self, session: &Session, updates: &mut SessionUpdates, key_event: KeyEvent){
        unsafe {
            if self.executable.can_call(self.on_key){
                //Rediculous call mechanism
                self.executable.map();
                let fn_ptr: *const usize = &self.on_key;
                (*(fn_ptr as *const fn(&Session, &mut SessionUpdates, KeyEvent)))(session, updates, key_event);
                self.executable.unmap();
            }
        }
    }

    fn on_mouse(&mut self, session: &Session, updates: &mut SessionUpdates, mouse_event: MouseEvent, allow_catch: bool) -> bool{
        unsafe {
            if self.executable.can_call(self.on_mouse){
                //Rediculous call mechanism
                self.executable.map();
                let fn_ptr: *const usize = &self.on_mouse;
                let ret = (*(fn_ptr as *const fn(&Session, &mut SessionUpdates, MouseEvent, bool) -> bool))(session, updates, mouse_event, allow_catch);
                self.executable.unmap();
                return ret;
            }
        }
        return false;
    }
}
