use alloc::boxed::*;

use core::mem::size_of;
use core::ptr;

use common::random::*;
use common::string::*;

use graphics::consolewindow::*;
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

#[macro_export]
macro_rules! print_color {
    ($text:expr, $color:expr) => ({
        console_window().print(&$text, $color);
        console_window().redraw();
    });
}

#[macro_export]
macro_rules! print {
    ($text:expr) => (print_color!($text, Color::new(224, 224, 224)));
}

#[macro_export]
macro_rules! println {
    ($text:expr) => ({
        print!($text);
        print!(&"\n".to_string());
    });
}

#[macro_export]
macro_rules! readln {
    () => (console_window().read());
}
