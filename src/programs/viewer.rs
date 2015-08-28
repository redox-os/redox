use graphics::bmp::*;
use graphics::size::*;
use graphics::window::*;

use programs::common::*;

pub struct Viewer {
    window: Window,
    events: Queue<Event>,
    closed: AtomicBool
}

impl Viewer {
    pub fn new() -> Viewer {
        Viewer {
            window: Window::new(Point::new((rand() % 400 + 50) as isize, (rand() % 300 + 50) as isize), Size::new(640, 480), "Viewer".to_string()),
            events: Queue::new(),
            closed: AtomicBool::new(false)
        }
    }
}

impl SessionItem for Viewer {
    fn main(&mut self, url: URL){
        let mut resource = url.open();

        let mut vec: Vec<u8> = Vec::new();
        resource.read_to_end(&mut vec);

        unsafe {
            let image = BMP::from_data(vec.as_ptr() as usize);
            self.window.size = image.size;
            self.window.content = Display::new(image.size.width, image.size.height);
            self.window.content.image(Point::new(0, 0), image.data, image.size);

            self.window.content.flip();

            RedrawEvent {
                redraw: REDRAW_ALL
            }.to_event().trigger();
        }

        self.window.title = "Viewer (".to_string() + url.to_string() + ")";

        RedrawEvent {
            redraw: REDRAW_ALL
        }.to_event().trigger();

        while ! self.closed.load(Ordering::SeqCst) {
            let event_option;
            unsafe{
                let enable = start_no_ints();
                event_option = self.events.pop();
                end_no_ints(enable);
            }

            match event_option {
                Option::Some(_) => (),
                Option::None => sys_yield()
            }
        }
    }

    fn draw(&self, display: &Display) -> bool{
        self.window.draw(display);
        return ! self.closed.load(Ordering::SeqCst);
    }

    fn on_key(&mut self, key_event: KeyEvent){
        if key_event.pressed && key_event.scancode == 1 {
            self.closed.store(true, Ordering::SeqCst);
        }
        self.events.push(key_event.to_event());
    }

    fn on_mouse(&mut self, mouse_event: MouseEvent, allow_catch: bool) -> bool{
        return self.window.on_mouse(mouse_event, allow_catch);
    }
}
