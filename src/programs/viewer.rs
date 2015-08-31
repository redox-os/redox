use graphics::bmp::*;

use programs::common::*;

pub struct Viewer;

impl Viewer {
    pub fn new() -> Viewer {
        Viewer
    }
}

impl SessionItem for Viewer {
    fn main(&mut self, url: URL){
        let mut resource = url.open();

        let mut vec: Vec<u8> = Vec::new();
        resource.read_to_end(&mut vec);

        let mut window = Window::new(Point::new((rand() % 400 + 50) as isize, (rand() % 300 + 50) as isize), Size::new(640, 480), "Viewer".to_string());

        unsafe {
            let image = BMP::from_data(vec.as_ptr() as usize);
            window.size = image.size;
            window.content = Display::new(image.size.width, image.size.height);
            window.content.image(Point::new(0, 0), image.data, image.size);

            window.content.flip();

            RedrawEvent {
                redraw: REDRAW_ALL
            }.to_event().trigger();

            window.title = "Viewer (".to_string() + url.to_string() + ")";
        }

        RedrawEvent {
            redraw: REDRAW_ALL
        }.to_event().trigger();

        loop {
            match window.poll() {
                EventOption::Key(key_event) => {
                    if key_event.pressed && key_event.scancode == 1 {
                        break;
                    }
                },
                EventOption::None => sys_yield(),
                _ => ()
            }
        }
    }
}
