use redox::*;

pub struct Viewer;

impl Viewer {
    pub fn new() -> Viewer {
        Viewer
    }

    fn main(&mut self, url: String){
        let mut resource = File::open(&url);

        let mut vec: Vec<u8> = Vec::new();
        resource.read_to_end(&mut vec);

        let mut window = Window::new(Point::new((rand() % 400 + 50) as isize, (rand() % 300 + 50) as isize), Size::new(640, 480), "Viewer".to_string());

        let image = BMP::from_data(&vec);
        window.size = image.size;
        window.content = Display::new(image.size.width, image.size.height);
        image.draw(&window.content, Point::new(0, 0));
        window.content.flip();

        window.title = "Viewer (".to_string() + &url + ")";

        RedrawEvent {
            redraw: REDRAW_ALL
        }.to_event().trigger();

        loop {
            match window.poll() {
                EventOption::Key(key_event) => {
                    if key_event.pressed && key_event.scancode == K_ESC {
                        break;
                    }
                },
                EventOption::None => sys_yield(),
                _ => ()
            }
        }
    }
}

pub fn main(){
    match args().get(1) {
        Option::Some(arg) => Viewer::new().main(arg.clone()),
        Option::None => ()
    }
}
