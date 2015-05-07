use common::debug::*;
use common::elf::*;
use common::string::*;

use drivers::disk::*;
use drivers::keyboard::*;
use drivers::mouse::*;

use filesystems::unfs::*;

use programs::program::*;

pub struct Executor {
    executable: ELF,
    entry: usize,
    draw: usize,
    on_key: usize,
    on_mouse: usize
}

impl Executor {
    pub unsafe fn new(file: &String) -> Executor {
        let mut ret = Executor {
            executable: ELF::new(),
            entry: 0,
            draw: 0,
            on_mouse: 0,
            on_key: 0
        };

        if file.len() > 0{
            d("Load executable file ");
            file.d();
            dl();

            ret.executable = ELF::from_data(UnFS::new(Disk::new()).load(file));
            //ret.executable.d();

            ret.entry = ret.executable.entry();
            ret.draw = ret.executable.symbol(&String::from_str("draw"));
            ret.on_key = ret.executable.symbol(&String::from_str("on_key"));
            ret.on_mouse = ret.executable.symbol(&String::from_str("on_mouse"));

            ret.entry();
        }

        return ret;
    }

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

impl Program for Executor {
    unsafe fn draw(&self, session: &mut Session) -> bool{
        if self.executable.can_call(self.draw){
            //Rediculous call mechanism
            self.executable.map();
            let fn_ptr: *const usize = &self.draw;
            let ret = (*(fn_ptr as *const fn(&mut Session) -> bool))(session);
            self.executable.unmap();

            return ret;
        }
        return false;
    }

    unsafe fn on_key(&mut self, session: &mut Session, key_event: KeyEvent){
        if self.executable.can_call(self.on_key){
            //Rediculous call mechanism
            self.executable.map();
            let fn_ptr: *const usize = &self.on_key;
            (*(fn_ptr as *const fn(&mut Session, KeyEvent)))(session, key_event);
            self.executable.unmap();
        }
    }

    unsafe fn on_mouse(&mut self, session: &mut Session, mouse_event: MouseEvent, allow_catch: bool) -> bool{
        if self.executable.can_call(self.on_mouse){
            //Rediculous call mechanism
            self.executable.map();
            let fn_ptr: *const usize = &self.on_mouse;
            let ret = (*(fn_ptr as *const fn(&mut Session, MouseEvent, bool) -> bool))(session, mouse_event, allow_catch);
            self.executable.unmap();
            return ret;
        }
        return false;
    }
}