use core::clone::Clone;

use common::string::*;
use common::url::*;

use drivers::keyboard::*;
use drivers::mouse::*;

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
    fn new() -> Viewer {
        Viewer {
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
        }
    }

    fn load(&mut self, session: &Session, filename: String){
        if filename.len() > 0{
            self.window.title = String::from_str("Viewer Loading (") + filename.clone() + String::from_str(")");

            self.image = BMP::new();

            session.on_url(&URL::from_string("file:///".to_string() + filename.clone()), box move |me: &mut SessionItem, response: String|{
                match me.downcast_mut::<Viewer>() {
                    Option::Some(viewer) => {
                        viewer.window.title = String::from_str("Viewer (") + filename.clone() + String::from_str(")");
                        if response.data as usize > 0 {
                            viewer.image = BMP::from_data(response.data as usize);
                            viewer.window.size = viewer.image.size;
                        }
                    },
                    Option::None => ()
                }
            });
        }
    }

    #[allow(unused_variables)]
    fn draw(&mut self, session: &Session, updates: &mut SessionUpdates) -> bool{
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
    fn on_key(&mut self, session: &Session, updates: &mut SessionUpdates, key_event: KeyEvent){
        if key_event.pressed {
            match key_event.scancode {
                0x01 => self.window.closed = true,
                _ => ()
            }
        }
    }

    #[allow(unused_variables)]
    fn on_mouse(&mut self, session: &Session, updates: &mut SessionUpdates, mouse_event: MouseEvent, allow_catch: bool) -> bool{
        return self.window.on_mouse(session.mouse_point, mouse_event, allow_catch);
    }
}
