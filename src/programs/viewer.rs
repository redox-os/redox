use graphics::bmp::*;
use graphics::size::*;
use graphics::window::*;

use programs::common::*;

pub struct Viewer {
    window: Window
}

impl SessionItem for Viewer {
    fn new() -> Viewer {
        Viewer {
            window: Window::new(Point::new((rand() % 400 + 50) as isize, (rand() % 300 + 50) as isize), Size::new(640, 480), "Viewer".to_string())
        }
    }

    fn load(&mut self, url: &URL){
        let mut resource = url.open();

        let mut vec: Vec<u8> = Vec::new();
        resource.read_to_end(&mut vec);

        unsafe {
            let image = BMP::from_data(vec.as_ptr() as usize);
            self.window.size = image.size;
            self.window.content = Display::new(image.size.width, image.size.height);
            content.image(Point::new(0, 0), image.data, image.size);
        }

        self.window.title = "Viewer (".to_string() + url.to_string() + ")";
    }

    fn draw(&mut self, display: &Display) -> bool{
        return self.window.draw(display);
    }

    fn on_key(&mut self, key_event: KeyEvent){
        if key_event.pressed {
            match key_event.scancode {
                0x01 => self.window.closed = true,
                _ => ()
            }
        }
    }

    fn on_mouse(&mut self, mouse_point: Point, mouse_event: MouseEvent, allow_catch: bool) -> bool{
        return self.window.on_mouse(mouse_point, mouse_event, allow_catch);
    }
}
