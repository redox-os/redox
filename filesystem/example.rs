use core::clone::Clone;
use core::option::Option;

use alloc::boxed::*;

use collections::vec::*;

use common::string::*;
use common::url::*;

use drivers::keyboard::*;
use drivers::mouse::*;

use graphics::color::*;
use graphics::point::*;
use graphics::size::*;
use graphics::window::*;

use programs::session::*;

use syscall;

pub struct Application {
    window: Window,
    output: String,
    last_command: String,
    command: String,
    offset: usize,
    scroll: Point,
    wrap: bool
}

impl Application {
    fn append(&mut self, line: String) {
        self.output = self.output.clone() + line + "\n";
    }

    #[allow(unused_variables)]
    fn on_command(&mut self, session: &Session){
        self.last_command = self.command.clone();
        let mut args: Vec<String> = Vec::<String>::new();
        for arg in self.command.split(" ".to_string()) {
            if arg.len() > 0 {
                args.push(arg);
            }
        }
        match args.get(0) {
            Option::Some(cmd) => {
                if *cmd == "break".to_string() {
                    unsafe{
                        asm!("int 3" : : : : "intel");
                    }
                }else if *cmd == "echo".to_string() {
                    let mut echo = String::new();
                    for i in 1..args.len() {
                        match args.get(i) {
                            Option::Some(arg) => {
                                if echo.len() == 0 {
                                    echo = arg.clone();
                                }else{
                                    echo = echo + " " + arg.clone();
                                }
                            },
                            Option::None => ()
                        }
                    }
                    self.append(echo);
                }else if *cmd == "exit".to_string() {
                    self.window.closed = true;
                }else if *cmd == "url".to_string() {
                    match args.get(1) {
                        Option::Some(url_string) => {
                            let url = URL::from_string(url_string.clone());
                            self.append(url.to_string());

                            let mut resource = syscall::open(&url);
                            loop {
                                let buf: &mut [u8] = &mut [0; 256];
                                match resource.read(buf){
                                    Option::Some(len) => {
                                        if len == 0 {
                                            break;
                                        }
                                        self.append(String::from_c_slice(buf));
                                    },
                                    Option::None => {
                                        self.append("Failed to read".to_string());
                                        break;
                                    }
                                }
                            }

                            /*
                            self.request(session, &url, box move |item: &mut SessionItem, response: String|{
                                response.d();
                                dl();

                                match item.downcast_mut::<Application>() {
                                    Option::Some(app) => {
                                        app.append(response);
                                    },
                                    Option::None => d("Failed to downcast application\n")
                                }
                            });
                            */
                        },
                        Option::None => {
                            for module in session.modules.iter() {
                                let scheme = module.scheme();
                                if scheme.len() > 0 {
                                    self.append(scheme);
                                }
                            }
                        }
                    }
                }else{
                    self.append("Commands:  echo  exit  url".to_string());
                }
            },
            Option::None => ()
        }
    }
}

impl SessionItem for Application {
    #[allow(unused_variables)]
    fn new() -> Application {
        Application {
            window: Window{
                point: Point::new(220, 100),
                size: Size::new(576, 400),
                title: String::from_str("Terminal"),
                title_color: Color::new(0, 0, 0),
                border_color: Color::new(192, 192, 255),
                content_color: Color::alpha(128, 128, 160, 192),
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
            output: String::new(),
            last_command: String::new(),
            command: String::new(),
            offset: 0,
            scroll: Point::new(0, 0),
            wrap: true
        }
    }

    fn draw(&mut self, session: &Session, updates: &mut SessionUpdates) -> bool{
        let display = &session.display;
        if self.window.draw(display) {
            let scroll = self.scroll;

            let mut col = -scroll.x;
            let cols = self.window.size.width as isize / 8;
            let mut row = -scroll.y;
            let rows = self.window.size.height as isize / 16;

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
                        let point = Point::new(self.window.point.x + 8 * col, self.window.point.y + 16 * row);
                        display.char(point, c, Color::new(224, 224, 224));
                    }
                    col += 1;
                }
            }

            if col > -scroll.x {
                col = -scroll.x;
                row += 1;
            }

            if col >= 0 && col < cols && row >= 0 && row < rows{
                let point = Point::new(self.window.point.x + 8 * col, self.window.point.y + 16 * row);
                display.char(point, '#', Color::new(255, 255, 255));
                col += 2;
            }

            let mut i = 0;
            for c in self.command.chars(){
                if self.wrap && col >= cols {
                    col = -scroll.x;
                    row += 1;
                }

                if self.offset == i && col >= 0 && col < cols && row >= 0 && row < rows{
                    let point = Point::new(self.window.point.x + 8 * col, self.window.point.y + 16 * row);
                    display.char(point, '_', Color::new(255, 255, 255));
                }

                if c == '\n' {
                    col = -scroll.x;
                    row += 1;
                }else if c == '\t' {
                    col += 8 - col % 8;
                }else{
                    if col >= 0 && col < cols && row >= 0 && row < rows{
                        let point = Point::new(self.window.point.x + 8 * col, self.window.point.y + 16 * row);
                        display.char(point, c, Color::new(255, 255, 255));
                    }
                    col += 1;
                }

                i += 1;
            }

            if self.wrap && col >= cols {
                col = -scroll.x;
                row += 1;
            }

            if row >= rows {
                self.scroll.y += row - rows + 1;
                updates.redraw = REDRAW_ALL;
            }

            if self.offset == i && col >= 0 && col < cols && row >= 0 && row < rows{
                let point = Point::new(self.window.point.x + 8 * col, self.window.point.y + 16 * row);
                display.char(point, '_', Color::new(255, 255, 255));
            }

            return true;
        }else{
            return false;
        }
    }

    #[allow(unused_variables)]
    fn on_key(&mut self, session: &Session, updates: &mut SessionUpdates, key_event: KeyEvent){
        if key_event.pressed {
            match key_event.scancode {
                0x01 => self.window.closed = true,
                0x47 => self.offset = 0,
                0x48 => {
                    self.command = self.last_command.clone();
                    self.offset = self.command.len();
                },
                0x4B => if self.offset > 0 {
                    self.offset -= 1;
                },
                0x4D => if self.offset < self.command.len() {
                    self.offset += 1;
                },
                0x4F => self.offset = self.command.len(),
                0x50 => {
                    self.command = String::new();
                    self.offset = self.command.len();
                },
                _ => ()
            }

            match key_event.character {
                '\x00' => (),
                '\x08' => {
                    if self.offset > 0 {
                        self.command = self.command.substr(0, self.offset - 1) + self.command.substr(self.offset, self.command.len() - self.offset);
                        self.offset -= 1;
                    }
                },
                '\n' => {
                    if self.command.len() > 0 {
                        self.output = self.output.clone() + "# ".to_string() + self.command.clone() + "\n";
                        self.on_command(session);
                        self.command = String::new();
                        self.offset = 0;
                    }
                },
                _ => {
                    self.command = self.command.substr(0, self.offset) + key_event.character + self.command.substr(self.offset, self.command.len() - self.offset);
                    self.offset += 1;
                }
            }
        }
    }

    #[allow(unused_variables)]
    fn on_mouse(&mut self, session: &Session, updates: &mut SessionUpdates, mouse_event: MouseEvent, allow_catch: bool) -> bool{
        return self.window.on_mouse(session.mouse_point, mouse_event, allow_catch);
    }

    #[allow(unused_variables)]
    fn request(&self, session: &Session, url: &URL, callback: Box<FnBox(&mut SessionItem, String)>) where Self:Sized{
        syscall::request(url, callback);
    }
}
