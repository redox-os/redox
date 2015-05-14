use core::clone::Clone;

use common::memory::*;
use common::string::*;

use drivers::disk::*;
use drivers::keyboard::*;
use drivers::mouse::*;

use filesystems::unfs::*;

use graphics::bmp::*;
use graphics::color::*;
use graphics::point::*;
use graphics::size::*;
use graphics::window::*;

use programs::session::*;

pub struct Viewer {
    window: Window,
    image: BMP
}

impl SessionItem for Viewer {
    fn new(file: String) -> Viewer {
        let mut ret = Viewer {
            window: Window{
                point: Point::new(180, 50),
                size: Size::new(640, 480),
                title: "Viewer".to_string(),
                title_color: Color::new(255, 255, 255),
                border_color: Color::new(0, 0, 0),
                content_color: Color::alpha(0, 0, 0, 0),
                shaded: false,
                closed: false,
                dragging: false,
                last_mouse_point: Point::new(0, 0),
                last_mouse_event: MouseEvent {
                    x: 0,
                    y: 0,
                    left_button: false,
                    right_button: false,
                    middle_button: false,
                    valid: false
                }
            },
            image: BMP::new()
        };

        if file.len() > 0{
            ret.window.title = String::from_str("Viewer (") + file.clone() + String::from_str(")");
            unsafe {
                let unfs = UnFS::new(Disk::new());
                let image_data = unfs.load(file.clone());
                if image_data > 0 {
                    ret.image = BMP::from_data(image_data);
                    ret.window.size = ret.image.size;
                    unalloc(image_data);
                }
            }
        }

        return ret;
    }

    unsafe fn draw(&mut self, session: &mut Session) -> bool{
        let display = &session.display;

        if ! self.window.draw(display) {
            return false;
        }

        if ! self.window.shaded {
            // TODO: Improve speed!
            if ! self.window.shaded {
                display.image(self.window.point, self.image.data, self.image.size);
            }
        }

        return true;
    }

    #[allow(unused_variables)]
    unsafe fn on_key(&mut self, session: &mut Session, key_event: KeyEvent){
        if key_event.pressed {
            match key_event.scancode {
                0x01 => self.window.closed = true,
                _ => ()
            }
        }
    }

    unsafe fn on_mouse(&mut self, session: &mut Session, mouse_event: MouseEvent, allow_catch: bool) -> bool{
        return self.window.on_mouse(session.mouse_point, mouse_event, allow_catch);
    }
}