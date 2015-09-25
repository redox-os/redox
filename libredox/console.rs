use alloc::boxed::*;

use core::mem::size_of;
use core::ptr;

use common::event::*;
use common::random::*;
use common::string::*;

use graphics::consolewindow::*;
use graphics::color::*;
use graphics::point::*;
use graphics::size::*;

use syscall::call::*;

static mut window: *mut Box<ConsoleWindow> = 0 as *mut Box<ConsoleWindow>;

pub fn console_window<'a>() -> &'a mut Box<ConsoleWindow> {
    unsafe{
        if window as usize == 0 {
            window = sys_alloc(size_of::<Box<ConsoleWindow>>()) as *mut Box<ConsoleWindow>;
            ptr::write(window, ConsoleWindow::new(Point::new((rand() % 400 + 50) as isize, (rand() % 300 + 50) as isize), Size::new(640, 480), "Console".to_string()));
            (*window).redraw();
        }
        return &mut *window;
    }
}

pub fn console_init() {
    unsafe{
        window = 0 as *mut Box<ConsoleWindow>;
    }
}

pub fn console_destroy() {
    unsafe{
        if window as usize > 0 {
            drop(ptr::read(window));
            sys_unalloc(window as usize);
            window = 0 as *mut Box<ConsoleWindow>;
        }
    }
}

pub fn console_title(title: &String){
    console_window().window.title = title.clone();
    console_window().redraw();
}

pub fn print_color(text: &String, color: Color){
    console_window().print(text, color);
    console_window().redraw();
}

pub fn print(text: &String){
    print_color(text, Color::new(224, 224, 224));
}

pub fn println(text: &String){
    print(text);
    print(&"\n".to_string());
}

#[macro_export]
macro_rules! println {
    ($text:expr) => ({
        println(&$text);
    });
}

pub fn readln() -> String {
    return console_window().read();
}

#[macro_export]
macro_rules! readln {
    () => (readln());
}
