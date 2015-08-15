use alloc::boxed::*;

use core::clone::Clone;

use common::resource::*;
use common::string::*;
use common::vec::*;

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
    image: BMP,
    loading: bool
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
            image: BMP::new(),
            loading: false
        }
    }

    fn load(&mut self, url: &URL){
        self.window.title = "Viewer Loading (".to_string() + url.to_string() + ")";

        self.image = BMP::new();
        self.loading = true;

        let self_ptr: *mut Viewer = self;
        let url_copy = url.clone();
        url.open_async(box move |mut resource: Box<Resource>|{
            let viewer;
            unsafe {
                viewer = &mut *self_ptr;
            }

            let mut vec: Vec<u8> = Vec::new();
            match resource.read_to_end(&mut vec){
                Option::Some(0) => (),
                Option::Some(len) => {
                    unsafe {
                        viewer.image = BMP::from_data(vec.as_ptr() as usize);
                    }
                    viewer.window.size = viewer.image.size;
                },
                Option::None => ()
            }

            viewer.window.title = "Viewer (".to_string() + url_copy.to_string() + ")";
            viewer.loading = false;
        });
    }

    #[allow(unused_variables)]
    fn draw(&mut self, session: &Session, updates: &mut SessionUpdates) -> bool{
        let display = &session.display;

        if ! self.window.draw(display) {
            return self.loading;
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
