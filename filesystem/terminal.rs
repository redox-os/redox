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
                        let path = arg.clone();
                        println!("URL: ".to_string() + &path);

                        let mut resource = File::open(&path);

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
                println!("URL: ".to_string() + &path);

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
                println!("URL: ".to_string() + &path);

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
                println!("URL: ".to_string() + &path);

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
    modes: Vec<Mode>
}

impl Application {
    pub fn new() -> Application {
        return Application {
            commands: Command::vec(),
            variables: Vec::new(),
            modes: Vec::new()
        };
    }

    fn on_command(&mut self, command_string: &String){
        //Comment
        if command_string[0] == '#' {
            return;
        }

        //Show variables
        if *command_string == "$".to_string() {
            let mut variables = String::new();
            for variable in self.variables.iter() {
                variables = variables + '\n' + &variable.name + "=" + &variable.value;
            }
            println!(&variables);
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
                                        println!(&("Unknown comparison: ".to_string() + cmp));
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
                        println!(&"Syntax error: else found with no previous if".to_string());
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
                        println!(&"Syntax error: fi found with no previous if".to_string());
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

                        if name.len() == 0 {
                            return;
                        }

                        for i in 1..args.len() {
                            match args.get(i) {
                                Option::Some(arg) => value = value + ' ' + arg.clone(),
                                Option::None => ()
                            }
                        }

                        if value.len() == 0 {
                            let mut remove = -1;
                            for i in 0..self.variables.len() {
                                match self.variables.get(i) {
                                    Option::Some(variable) => if variable.name == name {
                                        remove = i as isize;
                                        break;
                                    },
                                    Option::None => break
                                }
                            }

                            if remove >= 0 {
                                self.variables.remove(remove as usize);
                            }
                        }else{
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
                        }
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
                println!(&help);
            },
            Option::None => ()
        }
    }

    fn main(&mut self){
        console_title(&"Terminal".to_string());

        while let Option::Some(command) = readln!() {
            println!("# ".to_string() + &command);
            if command.len() > 0 {
                self.on_command(&command);
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
