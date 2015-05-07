use core::result::Result;

use common::memory::*;
use common::string::*;
use common::vector::*;

use drivers::keyboard::*;
use drivers::mouse::*;

use graphics::color::*;
use graphics::display::*;
use graphics::point::*;
use graphics::size::*;

pub trait Program {
    unsafe fn draw(&self, session: &mut Session) -> bool;
    unsafe fn on_key(&mut self, session: &mut Session, key_event: KeyEvent);
    unsafe fn on_mouse(&mut self, session: &mut Session, mouse_event: MouseEvent, alloc_catch: bool) -> bool;
}

pub struct Session {
    pub display: Display,
    pub mouse_point: Point,
    pub programs: Vector<Box<Program>>,
    pub draw: bool
}

impl Session {
    pub unsafe fn new() -> Session {
        Session {
            display: Display::new(),
            mouse_point: Point::new(0, 0),
            programs: Vector::<Box<Program>>::new(),
            draw: true
        }
    }

    // TODO: Find out how to remove
    pub fn copy_programs(&self) -> Vector<Box<Program>>{
        let mut ret: Vector<Box<Program>> = Vector::<Box<Program>>::new();
        for program in self.programs.as_slice() {
            ret = ret + Vector::<Box<Program>>::from_ptr(program);
        }
        return ret;
    }

    pub fn add_program(&mut self, program: Box<Program>){
        let mut new_programs = self.copy_programs();
        new_programs = Vector::<Box<Program>>::from_value(program) + new_programs;
        self.programs = new_programs;
        self.draw = true;
    }

    pub unsafe fn on_key(&mut self, key_event: KeyEvent){
        let programs = self.copy_programs();
        for program in programs.as_slice() {
            (*program).on_key(self, key_event);
            self.draw = true;
            break;
        }
    }

    pub unsafe fn on_mouse(&mut self, mouse_event: MouseEvent){
        self.mouse_point.x += mouse_event.x;
        if self.mouse_point.x < 0 {
            self.mouse_point.x = 0;
        }
        if self.mouse_point.x >= self.display.size.width as i32 {
            self.mouse_point.x = self.display.size.width as i32 - 1;
        }

        self.mouse_point.y += mouse_event.y;
        if self.mouse_point.y < 0 {
            self.mouse_point.y = 0;
        }
        if self.mouse_point.y >= self.display.size.height as i32 {
            self.mouse_point.y = self.display.size.height as i32 - 1;
        }

        let programs = self.copy_programs();
        let mut new_programs = Vector::<Box<Program>>::new();
        let mut allow_catch = true;
        for program in programs.as_slice() {
            if (*program).on_mouse(self, mouse_event, allow_catch) {
                new_programs = Vector::<Box<Program>>::from_ptr(program) + new_programs;
                allow_catch = false;
            }else{
                new_programs = new_programs + Vector::<Box<Program>>::from_ptr(program);
            }
        }
        self.programs = new_programs;

        self.draw = true;
    }

    pub unsafe fn redraw(&mut self){
        if self.draw {
            self.display.clear(Color::new(64, 64, 64));

            self.display.rect(Point::new(0, 0), Size::new(self.display.size.width, 18), Color::new(0, 0, 0));

            self.display.text(Point::new(self.display.size.width as i32/ 2 - 3*8, 1), &String::from_str("UberOS"), Color::new(255, 255, 255));

            let programs = self.copy_programs();
            let mut new_programs = Vector::<Box<Program>>::new();
            for i in 0..programs.len() {
                match programs.get(programs.len() - 1 - i) {
                    Result::Ok(program) => if (*program).draw(self){
                        new_programs = Vector::<Box<Program>>::from_ptr(program) + new_programs;
                    },
                    Result::Err(_) => ()
                }
            }
            self.programs = new_programs;

            self.display.char_bitmap(self.mouse_point, &MOUSE_CURSOR as *const u8, Color::new(255, 255, 255));

            self.display.copy();

            self.draw = false;
        }
    }
}
