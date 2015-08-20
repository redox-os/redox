use graphics::bmp::*;
use graphics::color::*;
use graphics::size::*;
use graphics::window::*;

use programs::common::*;

pub struct Viewer {
    window: Window,
    image: BMP
}

impl SessionItem for Viewer {
    fn new() -> Viewer {
        Viewer {
            window: Window{
                point: Point::new((rand() % 400 + 50) as isize, (rand() % 300 + 50) as isize),
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

    fn load(&mut self, url: &URL){
        let mut resource = url.open();

        let mut vec: Vec<u8> = Vec::new();
        resource.read_to_end(&mut vec);

        unsafe {
            self.image = BMP::from_data(vec.as_ptr() as usize);
        }
        self.window.size = self.image.size;
        if self.window.size.width < 100 {
            self.window.size.width = 100;
        }

        self.window.title = "Viewer (".to_string() + url.to_string() + ")";
    }

    #[allow(unused_variables)]
    fn draw(&mut self, display: &Display) -> bool{
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
    fn on_key(&mut self, key_event: KeyEvent){
        if key_event.pressed {
            match key_event.scancode {
                0x01 => self.window.closed = true,
                _ => ()
            }
        }
    }

    #[allow(unused_variables)]
    fn on_mouse(&mut self, mouse_point: Point, mouse_event: MouseEvent, allow_catch: bool) -> bool{
        return self.window.on_mouse(mouse_point, mouse_event, allow_catch);
    }
}
