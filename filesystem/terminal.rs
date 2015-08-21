use graphics::color::*;
use graphics::size::*;
use graphics::window::*;

use programs::common::*;

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

    fn on_command(&mut self, command: &String){
        let mut args: Vec<String> = Vec::<String>::new();
        for arg in command.split(" ".to_string()) {
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
                }else if *cmd == "open".to_string() {
                    match args.get(1) {
                        Option::Some(arg) => OpenEvent{ url_string: arg.clone() }.trigger(),
                        Option::None => ()
                    }
                }else if *cmd == "run".to_string() {
                    match args.get(1) {
                        Option::Some(arg) => {
                            let mut resource = URL::from_string(arg.clone()).open();

                            let mut vec: Vec<u8> = Vec::new();
                            resource.read_to_end(&mut vec);

                            let commands = String::from_utf8(&vec);
                            for command in commands.split("\n".to_string()) {
                                self.on_command(&command);
                            }
                        },
                        Option::None => ()
                    }
                }else if *cmd == "url".to_string() {
                    let mut url = URL::new();

                    match args.get(1) {
                        Option::Some(arg) => url = URL::from_string(arg.clone()),
                        Option::None => ()
                    }

                    self.append("URL: ".to_string() + url.to_string());

                    let mut resource = url.open();

                    match resource.stat() {
                        ResourceType::File => self.append("Type: File".to_string()),
                        ResourceType::Dir => self.append("Type: Dir".to_string()),
                        ResourceType::Array => self.append("Type: Array".to_string()),
                        _ => self.append("Type: None".to_string())
                    }

                    let mut vec: Vec<u8> = Vec::new();
                    match resource.read_to_end(&mut vec) {
                        Option::Some(_) => self.append(String::from_utf8(&vec)),
                        Option::None => self.append("Failed to read".to_string())
                    }
                }else{
                    self.append("Commands:  echo  exit  open  run  url".to_string());
                }
            },
            Option::None => ()
        }
    }

    fn draw_content(&mut self){
        let scroll = self.scroll;

        let mut col = -scroll.x;
        let cols = self.window.content.width as isize / 8;
        let mut row = -scroll.y;
        let rows = self.window.content.height as isize / 16;

        {
            let content = &self.window.content;

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

            if col >= 0 && col < cols && row >= 0 && row < rows{
                content.char(Point::new(8 * col, 16 * row), '#', Color::new(255, 255, 255));
                col += 2;
            }

            let mut i = 0;
            for c in self.command.chars(){
                if self.wrap && col >= cols {
                    col = -scroll.x;
                    row += 1;
                }

                if self.offset == i && col >= 0 && col < cols && row >= 0 && row < rows{
                    content.char(Point::new(8 * col, 16 * row), '_', Color::new(255, 255, 255));
                }

                if c == '\n' {
                    col = -scroll.x;
                    row += 1;
                }else if c == '\t' {
                    col += 8 - col % 8;
                }else{
                    if col >= 0 && col < cols && row >= 0 && row < rows{
                        content.char(Point::new(8 * col, 16 * row), c, Color::new(255, 255, 255));
                    }
                    col += 1;
                }

                i += 1;
            }

            if self.wrap && col >= cols {
                col = -scroll.x;
                row += 1;
            }

            if self.offset == i && col >= 0 && col < cols && row >= 0 && row < rows{
                content.char(Point::new(8 * col, 16 * row), '_', Color::new(255, 255, 255));
            }
        }

        if row >= rows {
            self.scroll.y += row - rows + 1;

            self.draw_content();
            RedrawEvent {
                redraw: REDRAW_ALL
            }.trigger();
        }
    }
}

impl SessionItem for Application {
    fn new() -> Application {
        let mut ret = Application {
            window: Window::new(Point::new((rand() % 400 + 50) as isize, (rand() % 300 + 50) as isize), Size::new(576, 400), String::from_str("Terminal")),
            output: String::new(),
            last_command: String::new(),
            command: String::new(),
            offset: 0,
            scroll: Point::new(0, 0),
            wrap: true
        };

        ret.draw_content();

        return ret;
    }

    fn draw(&self, display: &Display) -> bool{
        return self.window.draw(display);
    }

    fn on_key(&mut self, key_event: KeyEvent){
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
                        let command = self.command.clone();
                        self.command = String::new();
                        self.offset = 0;
                        self.last_command = command.clone();
                        self.output = self.output.clone() + "# ".to_string() + command.clone() + "\n";
                        self.on_command(&command);
                    }
                },
                _ => {
                    self.command = self.command.substr(0, self.offset) + key_event.character + self.command.substr(self.offset, self.command.len() - self.offset);
                    self.offset += 1;
                }
            }

            self.draw_content();
        }
    }

    fn on_mouse(&mut self, mouse_point: Point, mouse_event: MouseEvent, allow_catch: bool) -> bool{
        if self.window.on_mouse(mouse_point, mouse_event, allow_catch){
            self.draw_content();
            return true;
        }else{
            return false;
        }
    }
}
