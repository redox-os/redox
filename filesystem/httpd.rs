use programs::common::*;

pub struct Application {
    output: String,
    scroll: Point,
    wrap: bool
}

impl SessionItem for Application {
    fn main(&mut self, url: URL){
        let mut window = Window::new(Point::new((rand() % 400 + 50) as isize, (rand() % 300 + 50) as isize), Size::new(576, 400), "HTTP Server".to_string());

        macro_rules! println {
            ($text:expr) => ({
                self.output.vec.push_all(&$text.vec);
                self.output.vec.push('\n');
                self.draw_content(&mut window);
            });
        }

        println!("Starting HTTP Server".to_string());

        match TcpListener::bind(80){
            Result::Ok(mut listener) => {
                println!("Listening for connections".to_string());
                loop {
                    match listener.poll() {
                        Option::Some(mut stream) => {
                            println!("Incoming stream from ".to_string() + stream.address.to_string() + ":" + stream.port as usize);
                            println!(String::from_utf8(&stream.data));
                            stream.response = "Test".to_string().to_utf8();
                        }
                        Option::None => ()
                    }

                    match window.poll() {
                        EventOption::Key(key_event) => {
                            if key_event.pressed{
                                if key_event.scancode == 1 {
                                    break;
                                }
                            }
                        },
                        EventOption::None => sys_yield(),
                        _ => ()
                    }
                }
                println!("Stopped listening for connections".to_string());
            },
            Result::Err(e) => println!(e)
        }

        println!("Closed HTTP Server".to_string());

        loop {
            match window.poll() {
                EventOption::Key(key_event) => {
                    if key_event.pressed{
                        if key_event.scancode == 1 {
                            break;
                        }
                    }
                },
                EventOption::None => sys_yield(),
                _ => ()
            }
        }
    }
}

impl Application {
    pub fn new() -> Application {
        return Application {
            output: String::new(),
            scroll: Point::new(0, 0),
            wrap: true
        };
    }

    fn draw_content(&mut self, window: &mut Window){
        let scroll = self.scroll;

        let mut col = -scroll.x;
        let cols = window.content.width as isize / 8;
        let mut row = -scroll.y;
        let rows = window.content.height as isize / 16;

        {
            let content = &window.content;
            content.set(Color::new(0, 0, 0));

            for c in self.output.chars(){
                if self.wrap && col >= cols {
                    col = -scroll.x;
                    row += 1;
                }

                if c == '\n' {
                    col = -scroll.x;
                    row += 1;
                }else if c == '\t' {
                    col += 8 - col % 8;
                }else{
                    if col >= 0 && col < cols && row >= 0 && row < rows{
                        content.char(Point::new(8 * col, 16 * row), c, Color::new(224, 224, 224));
                    }
                    col += 1;
                }
            }

            if col > -scroll.x {
                col = -scroll.x;
                row += 1;
            }

            if self.wrap && col >= cols {
                col = -scroll.x;
                row += 1;
            }

            content.flip();

            RedrawEvent {
                redraw: REDRAW_ALL
            }.to_event().trigger();
        }

        if row >= rows {
            self.scroll.y += row - rows + 1;

            self.draw_content(window);
        }
    }
}
