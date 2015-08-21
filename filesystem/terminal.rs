use graphics::color::*;
use graphics::size::*;
use graphics::window::*;

use programs::common::*;

/* Magic Macros { */
use super::application;

macro_rules! exec {
    ($cmd:expr) => ({
        unsafe {
            (*application).on_command(&$cmd);
        }
    })
}

macro_rules! print {
    ($text:expr) => ({
        unsafe {
            (*application).stdio.write_all(&$text.to_utf8());
        }
    });
}

macro_rules! println {
    ($line:expr) => (print!($line + "\n"));
}
/* } Magic Macros */

pub struct Command {
    pub name: String,
    pub main: Box<Fn(&Vec<String>)>
}

impl Command {
    fn vec() -> Vec<Command> {
        let mut commands: Vec<Command> = Vec::new();

        commands.push(Command {
            name: "break".to_string(),
            main: box |args: &Vec<String>|{
                unsafe{
                    asm!("int 3" : : : : "intel");
                }
            }
        });

        commands.push(Command {
            name: "echo".to_string(),
            main: box |args: &Vec<String>|{
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
                println!(echo);
            }
        });

        commands.push(Command {
            name: "open".to_string(),
            main: box |args: &Vec<String>|{
                match args.get(1) {
                    Option::Some(arg) => OpenEvent{ url_string: arg.clone() }.trigger(),
                    Option::None => ()
                }
            }
        });

        commands.push(Command {
            name: "run".to_string(),
            main: box |args: &Vec<String>|{
                match args.get(1) {
                    Option::Some(arg) => {
                        let mut resource = URL::from_string(arg.clone()).open();

                        let mut vec: Vec<u8> = Vec::new();
                        resource.read_to_end(&mut vec);

                        let commands = String::from_utf8(&vec);
                        for command in commands.split("\n".to_string()) {
                            exec!(command);
                        }
                    },
                    Option::None => ()
                }
            }
        });

        commands.push(Command {
            name: "url".to_string(),
            main: box |args: &Vec<String>|{
                let mut url = URL::new();

                match args.get(1) {
                    Option::Some(arg) => url = URL::from_string(arg.clone()),
                    Option::None => ()
                }

                println!("URL: ".to_string() + url.to_string());

                let mut resource = url.open();

                match resource.stat() {
                    ResourceType::File => println!("Type: File".to_string()),
                    ResourceType::Dir => println!("Type: Dir".to_string()),
                    ResourceType::Array => println!("Type: Array".to_string()),
                    _ => println!("Type: None".to_string())
                }

                let mut vec: Vec<u8> = Vec::new();
                match resource.read_to_end(&mut vec) {
                    Option::Some(_) => println!(String::from_utf8(&vec)),
                    Option::None => println!("Failed to read".to_string())
                }
            }
        });

        return commands;
    }
}

pub struct Application {
    window: Window,
    commands: Vec<Command>,
    stdio: Box<VecResource>,
    last_command: String,
    command: String,
    offset: usize,
    scroll: Point,
    wrap: bool
}

impl Application {
    fn append(&mut self, line: String) {
        self.stdio.write_all(&(line + "\n").to_utf8());
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
                if cmd[0] == '#' {
                    return;
                }

                for command in self.commands.iter() {
                    if command.name == *cmd {
                        (*command.main)(&args);
                        return;
                    }
                }

                let mut help = "Commands:".to_string();
                for command in self.commands.iter() {
                    help = help + " " + command.name.clone();
                }

                self.append(help);
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

            let output = String::from_utf8(self.stdio.inner());
            for c in output.chars(){
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
        }
    }
}

impl SessionItem for Application {
    fn new() -> Application {
        let mut ret = Application {
            window: Window::new(Point::new((rand() % 400 + 50) as isize, (rand() % 300 + 50) as isize), Size::new(576, 400), String::from_str("Terminal")),
            commands: Command::vec(),
            stdio: box VecResource::new(ResourceType::File, Vec::new()),
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
                        self.append("# ".to_string() + command.clone());
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
