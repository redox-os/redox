use graphics::bmp::*;
use graphics::size::*;
use graphics::window::*;

use programs::common::*;

pub struct Viewer {
    window: Window,
    events: Queue<Event>
}

impl Viewer {
    pub fn new() -> Viewer {
        Viewer {
            window: Window::new(Point::new((rand() % 400 + 50) as isize, (rand() % 300 + 50) as isize), Size::new(640, 480), "Viewer".to_string()),
            events: Queue::new()
        }
    }
}

impl SessionItem for Viewer {
    fn main(&mut self){
        loop {
            let event_option;
            unsafe{
                let enable = start_no_ints();

                event_option = self.events.pop();

                end_no_ints(enable);
            }

            match event_option {
                Option::Some(event_const) => {
                    let mut event = event_const;
                    match event.code {
                        'm' => {
                            let mouse_event = MouseEvent::from_event(&mut event);
                            self.window.on_mouse(mouse_event, true);
                        },
                        'k' => {
                            let key_event = KeyEvent::from_event(&mut event);
                            if key_event.pressed {
                                match key_event.scancode {
                                    0x01 => self.window.closed = true,
                                    _ => ()
                                }
                            }
                        },
                        _ => ()
                    }
                },
                Option::None => sched_yield()
            }
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
            self.window.content.image(Point::new(0, 0), image.data, image.size);
        }

        self.window.title = "Viewer (".to_string() + url.to_string() + ")";
    }

    fn draw(&self, display: &Display) -> bool{
        return self.window.draw(display);
    }

    fn on_key(&mut self, key_event: KeyEvent){
        self.events.push(key_event.to_event());
    }

    fn on_mouse(&mut self, mouse_event: MouseEvent, allow_catch: bool) -> bool{
        self.events.push(mouse_event.to_event());
        return true;
    }
}
