use core::ops::DerefMut;

use redox::*;

/* Magic Macros { */
static mut application: *mut Application = 0 as *mut Application;

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
            (*application).print(&$text);
        }
    });
}

macro_rules! println {
    ($text:expr) => ({
        unsafe {
            (*application).println(&$text);
        }
    });
}
/* } Magic Macros */

pub struct Command {
    pub name: String,
    pub main: Box<Fn(&Vec<String>)>
}

impl Command {
    pub fn vec() -> Vec<Command> {
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
                let mut first = true;
                for i in 1..args.len() {
                    match args.get(i) {
                        Option::Some(arg) => {
                            if first {
                                first = false
                            }else{
                                echo = echo + " ";
                            }
                            echo = echo + arg;
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
                        println!(arg);

                        let mut resource = File::open(&arg);

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
            name: "send".to_string(),
            main: box |args: &Vec<String>|{
                let path;
                match args.get(1) {
                    Option::Some(arg) => path = arg.clone(),
                    Option::None => path = String::new()
                }
                println!(path);

                let mut resource = File::open(&path);

                let mut vec: Vec<u8> = Vec::new();
                for i in 2..args.len() {
                    match args.get(i) {
                        Option::Some(arg) => {
                            if i == 2 {
                                vec.push_all(&arg.to_utf8())
                            }else{
                                vec.push_all(&(" ".to_string() + arg).to_utf8())
                            }
                        },
                        Option::None => vec = Vec::new()
                    }
                }
                vec.push_all(&"\r\n\r\n".to_string().to_utf8());

                match resource.write(&vec.as_slice()) {
                    Option::Some(size) => println!("Wrote ".to_string() + size + " bytes"),
                    Option::None => println!("Failed to write".to_string())
                }

                vec = Vec::new();
                match resource.read_to_end(&mut vec) {
                    Option::Some(size) => println!(String::from_utf8(&vec)),
                    Option::None => println!("Failed to read".to_string())
                }
            }
        });

        commands.push(Command {
            name: "url".to_string(),
            main: box |args: &Vec<String>|{
                let path;
                match args.get(1) {
                    Option::Some(arg) => path = arg.clone(),
                    Option::None => path = String::new()
                }
                println!(path);

                let mut resource = File::open(&path);

                let mut vec: Vec<u8> = Vec::new();
                match resource.read_to_end(&mut vec) {
                    Option::Some(_) => println!(String::from_utf8(&vec)),
                    Option::None => println!("Failed to read".to_string())
                }
            }
        });

        commands.push(Command {
            name: "url_hex".to_string(),
            main: box |args: &Vec<String>|{
                let path;
                match args.get(1) {
                    Option::Some(arg) => path = arg.clone(),
                    Option::None => path = String::new()
                }
                println!(path);

                let mut resource = File::open(&path);

                let mut vec: Vec<u8> = Vec::new();
                match resource.read_to_end(&mut vec) {
                    Option::Some(_) => {
                        let mut line = "HEX:".to_string();
                        for byte in vec.iter() {
                            line = line + ' ' + String::from_num_radix(*byte as usize, 16);
                        }
                        println!(line);
                    },
                    Option::None => println!("Failed to read".to_string())
                }
            }
        });

        return commands;
    }
}

pub struct Variable {
    pub name: String,
    pub value: String
}

pub struct Mode {
    value: bool
}

pub struct Application {
    commands: Vec<Command>,
    variables: Vec<Variable>,
    modes: Vec<Mode>,
    output: String,
    last_command: String,
    command: String,
    offset: usize,
    scroll: Point,
    wrap: bool
}

impl Application {
    pub fn new() -> Application {
        return Application {
            commands: Command::vec(),
            variables: Vec::new(),
            modes: Vec::new(),
            output: String::new(),
            last_command: String::new(),
            command: String::new(),
            offset: 0,
            scroll: Point::new(0, 0),
            wrap: true
        };
    }

    fn print(&mut self, text: &String){
        self.output.vec.push_all(&text.vec);
    }

    fn println(&mut self, text: &String){
        self.print(text);
        self.output.vec.push('\n');
    }

    fn on_command(&mut self, command_string: &String){
        //Comment
        if command_string[0] == '#' {
            return;
        }

        //Show variables
        if *command_string == "$".to_string() {
            let mut variables = "Variables:".to_string();
            for variable in self.variables.iter() {
                variables = variables + '\n' + &variable.name + "=" + &variable.value;
            }
            self.println(&variables);
            return;
        }

        //Explode into arguments, replace variables
        let mut args: Vec<String> = Vec::<String>::new();
        for arg in command_string.split(" ".to_string()) {
            if arg.len() > 0 {
                if arg[0] == '$' {
                    let name = arg.substr(1, arg.len() - 1);
                    for variable in self.variables.iter() {
                        if variable.name == name {
                            args.push(variable.value.clone());
                            break;
                        }
                    }
                }else{
                    args.push(arg);
                }
            }
        }

        //Execute commands
        match args.get(0) {
            Option::Some(cmd) => {
                if *cmd == "if".to_string() {
                    let mut value = false;

                    match args.get(1) {
                        Option::Some(left) => match args.get(2) {
                            Option::Some(cmp) => match args.get(3) {
                                Option::Some(right) => {
                                    if *cmp == "==".to_string() {
                                        value = *left == *right;
                                    }else if *cmp == "!=".to_string() {
                                        value = *left != *right;
                                    }else if *cmp == ">".to_string() {
                                        value = left.to_num_signed() > right.to_num_signed();
                                    }else if *cmp == ">=".to_string() {
                                        value = left.to_num_signed() >= right.to_num_signed();
                                    }else if *cmp == "<".to_string() {
                                        value = left.to_num_signed() < right.to_num_signed();
                                    }else if *cmp == "<=".to_string() {
                                        value = left.to_num_signed() <= right.to_num_signed();
                                    }else{
                                        self.println(&("Unknown comparison: ".to_string() + cmp));
                                    }
                                },
                                Option::None => ()
                            },
                            Option::None => ()
                        },
                        Option::None => ()
                    }

                    self.modes.insert(0, Mode{
                        value: value
                    });
                    return;
                }

                if *cmd == "else".to_string() {
                    let mut syntax_error = false;
                    match self.modes.get(0) {
                        Option::Some(mode) => mode.value = !mode.value,
                        Option::None => syntax_error = true
                    }
                    if syntax_error {
                        self.println(&"Syntax error: else found with no previous if".to_string());
                    }
                    return;
                }

                if *cmd == "fi".to_string() {
                    let mut syntax_error = false;
                    match self.modes.remove(0) {
                        Option::Some(_) => (),
                        Option::None => syntax_error = true
                    }
                    if syntax_error {
                        self.println(&"Syntax error: fi found with no previous if".to_string());
                    }
                    return;
                }

                for mode in self.modes.iter() {
                    if ! mode.value {
                        return;
                    }
                }

                //Set variables
                match cmd.find("=".to_string()) {
                    Option::Some(i) => {
                        let name = cmd.substr(0, i);
                        let mut value = cmd.substr(i + 1, cmd.len() - i - 1);

                        for i in 1..args.len() {
                            match args.get(i) {
                                Option::Some(arg) => value = value + ' ' + arg.clone(),
                                Option::None => ()
                            }
                        }

                        for variable in self.variables.iter() {
                            if variable.name == name {
                                variable.value = value;
                                return;
                            }
                        }

                        self.variables.push(Variable{
                            name: name,
                            value: value
                        });
                        return;
                    },
                    Option::None => ()
                }

                //Commands
                for command in self.commands.iter() {
                    if command.name == *cmd {
                        (*command.main)(&args);
                        return;
                    }
                }

                let mut help = "Commands:".to_string();
                for command in self.commands.iter() {
                    help = help + ' ' + &command.name;
                }
                self.println(&help);
            },
            Option::None => ()
        }
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

        window.redraw();

        if row >= rows {
            self.scroll.y += row - rows + 1;

            self.draw_content(window);
        }
    }

    fn on_key(&mut self, key_event: KeyEvent){
        match key_event.scancode {
            K_BKSP => if self.offset > 0 {
                self.command = self.command.substr(0, self.offset - 1) + self.command.substr(self.offset, self.command.len() - self.offset);
                self.offset -= 1;
            },
            K_DEL => if self.offset < self.command.len() {
                self.command = self.command.substr(0, self.offset) + self.command.substr(self.offset + 1, self.command.len() - self.offset - 1);
            },
            K_HOME => self.offset = 0,
            K_UP => {
                self.command = self.last_command.clone();
                self.offset = self.command.len();
            },
            K_LEFT => if self.offset > 0 {
                self.offset -= 1;
            },
            K_RIGHT => if self.offset < self.command.len() {
                self.offset += 1;
            },
            K_END => self.offset = self.command.len(),
            K_DOWN => {
                self.command = String::new();
                self.offset = self.command.len();
            },
            _ => match key_event.character {
                '\x00' => (),
                '\n' => if self.command.len() > 0 {
                    let command = self.command.clone();
                    self.command = String::new();
                    self.offset = 0;
                    self.last_command = command.clone();
                    self.println(&("# ".to_string() + &command));
                    self.on_command(&command);
                },
                _ => {
                    self.command = self.command.substr(0, self.offset) + key_event.character + self.command.substr(self.offset, self.command.len() - self.offset);
                    self.offset += 1;
                }
            }
        }
    }

    fn main(&mut self){
        let mut window = Window::new(Point::new((rand() % 400 + 50) as isize, (rand() % 300 + 50) as isize), Size::new(576, 400), "Terminal".to_string());
        self.draw_content(&mut window);

        loop {
            match window.poll() {
                EventOption::Key(key_event) => {
                    if key_event.pressed{
                        if key_event.scancode == K_ESC {
                            break;
                        }

                        self.on_key(key_event);
                        self.draw_content(&mut window);
                    }
                },
                EventOption::None => sys_yield(),
                _ => ()
            }
        }
    }
}

pub fn main(){
    unsafe {
        let mut app = box Application::new();
        application = app.deref_mut();
        app.main();
    }
}
