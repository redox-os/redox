use redox::ops::DerefMut;
use redox::string::*;
use redox::vec::Vec;
use redox::boxed::Box;
use redox::fs::file::*;
use redox::io::*;
use redox::console::*;
use redox::env::*;
use redox::to_num::*;
use redox::{Color};

/* Magic Macros { */
static mut application: *mut Application = 0 as *mut Application;

/// Execute a command
macro_rules! exec {
    ($cmd:expr) => ({
        unsafe {
            (*application).on_command(&$cmd.to_string());
        }
    })
}
/* } Magic Macros */

/// A command
pub struct Command {
    pub name: String,
    pub main: Box<Fn(&Vec<String>)>,
}

impl Command {
    /// Return the vector of the commands
    // TODO: Use a more efficient collection instead
    pub fn vec() -> Vec<Self> {
        let mut commands: Vec<Self> = Vec::new();
        commands.push(Command {
            name: "echo".to_string(),
            main: box |args: &Vec<String>| {
                let mut echo = String::new();
                let mut first = true;
                for i in 1..args.len() {
                    match args.get(i) {
                        Some(arg) => {
                            if first {
                                first = false
                            } else {
                                echo = echo + " ";
                            }
                            echo = echo + arg;
                        }
                        None => (),
                    }
                }
                println!("{}", echo);
            },
        });

        commands.push(Command {
            name: "open".to_string(),
            main: box |args: &Vec<String>| {
                match args.get(1) {
                    Some(arg) => {
                        File::exec(arg);
                    },
                    None => (),
                }
            },
        });

        commands.push(Command {
            name: "run".to_string(),
            main: box |args: &Vec<String>| {
                match args.get(1) {
                    Some(arg) => {
                        let path = arg.clone();
                        println!("URL: {}", path);

                        let mut commands = String::new();
                        if let Some(mut file) = File::open(&path) {
                            file.read_to_string(&mut commands);
                        }

                        for command in commands.split('\n') {
                            exec!(command);
                        }
                    }
                    None => (),
                }
            },
        });

        commands.push(Command {
            name: "send".to_string(),
            main: box |args: &Vec<String>| {
                let path;
                match args.get(1) {
                    Some(arg) => path = arg.clone(),
                    None => path = String::new(),
                }
                println!("URL: {}", path);

                if let Some(mut file) = File::open(&path) {
                    let mut string = String::new();
                    for i in 2..args.len() {
                        if let Some(arg) = args.get(i) {
                            if i >= 3 {
                                string.push_str(" ");
                            }
                            string.push_str(arg);
                        }
                    }
                    string.push_str("\r\n\r\n");

                    match file.write(&string.as_bytes()) {
                        Some(size) => println!("Wrote {} bytes", size),
                        None => println!("Failed to write"),
                    }

                    string = String::new();
                    match file.read_to_string(&mut string) {
                        Some(_) => println!("{}", string),
                        None => println!("Failed to read"),
                    }
                }
            },
        });

        commands.push(Command {
            name: "url".to_string(),
            main: box |args: &Vec<String>| {
                let path;
                match args.get(1) {
                    Some(arg) => path = arg.clone(),
                    None => path = String::new(),
                }
                println!("URL: {}", path);

                if let Some(mut file) = File::open(&path) {
                    let mut string = String::new();
                    match file.read_to_string(&mut string) {
                        Some(_) => println!("{}", string),
                        None => println!("Failed to read"),
                    }
                }
            },
        });

        commands.push(Command {
            name: "url_hex".to_string(),
            main: box |args: &Vec<String>| {
                let path;
                match args.get(1) {
                    Some(arg) => path = arg.clone(),
                    None => path = String::new(),
                }
                println!("URL: {}", path);

                if let Some(mut file) = File::open(&path) {
                    let mut vec: Vec<u8> = Vec::new();
                    match file.read_to_end(&mut vec) {
                        Some(_) => {
                            let mut line = "HEX:".to_string();
                            for byte in vec.iter() {
                                line = line + " " + &format!("{:X}", *byte);
                            }
                            println!("{}", line);
                        }
                        None => println!("Failed to read"),
                    }
                }
            },
        });

        commands.push(Command {
            name: "wget".to_string(),
            main: box |args: &Vec<String>| {
                if let Some(host) = args.get(1) {
                    if let Some(req) = args.get(2) {
                        if let Some(mut con) = File::open(&("tcp://".to_string() + host)) {
                            con.write(("GET ".to_string() + req + " HTTP/1.1").as_bytes());

                            let mut res = Vec::new();
                            con.read_to_end(&mut res);

                            if let Some(mut file) = File::open(&req) {
                                file.write(&res);
                            }
                        }
                    } else {
                        println!("No request given");
                    }
                } else {
                    println!("No url given");
                }
            },
        });

        let mut command_list = String::new();
        command_list = commands.iter().fold(command_list, |l , c| l + " " + &c.name);

        commands.push(Command {
            name: "help".to_string(),
            main: box move |args: &Vec<String>| {
                println!("Commands:{}", command_list);
            },
         });

        commands
    }
}

/// A (env) variable
pub struct Variable {
    pub name: String,
    pub value: String,
}

pub struct Mode {
    value: bool,
}

/// An application
pub struct Application {
    commands: Vec<Command>,
    variables: Vec<Variable>,
    modes: Vec<Mode>,
}

impl Application {
    /// Create a new empty application
    pub fn new() -> Self {
        return Application {
            commands: Command::vec(),
            variables: Vec::new(),
            modes: Vec::new(),
        };
    }

    fn on_command(&mut self, command_string: &String) {
        //Comment
        if command_string.starts_with('#') {
            return;
        }

        //Show variables
        if *command_string == "$" {
            let mut variables = String::new();
            for variable in self.variables.iter() {
                variables = variables + "\n" + &variable.name + "=" + &variable.value;
            }
            println!("{}", variables);
            return;
        }

        //Explode into arguments, replace variables
        let mut args: Vec<String> = Vec::<String>::new();
        for arg in command_string.split(' ') {
            if arg.len() > 0 {
                if arg.starts_with('$') {
                    let name = arg[1 .. arg.len()].to_string();
                    for variable in self.variables.iter() {
                        if variable.name == name {
                            args.push(variable.value.clone());
                            break;
                        }
                    }
                } else {
                    args.push(arg.to_string());
                }
            }
        }

        //Execute commands
        match args.get(0) {
            Some(cmd) => {
                if cmd == "if" {
                    let mut value = false;

                    match args.get(1) {
                        Some(left) => match args.get(2) {
                            Some(cmp) => match args.get(3) {
                                Some(right) => {
                                    if cmp == "==" {
                                        value = *left == *right;
                                    } else if cmp == "!=" {
                                        value = *left != *right;
                                    } else if cmp == ">" {
                                        value = left.to_num_signed() > right.to_num_signed();
                                    } else if cmp == ">=" {
                                        value = left.to_num_signed() >= right.to_num_signed();
                                    } else if cmp == "<" {
                                        value = left.to_num_signed() < right.to_num_signed();
                                    } else if cmp == "<=" {
                                        value = left.to_num_signed() <= right.to_num_signed();
                                    } else {
                                        println!("Unknown comparison: {}", cmp);
                                    }
                                }
                                None => (),
                            },
                            None => (),
                        },
                        None => (),
                    }

                    self.modes.insert(0, Mode { value: value });
                    return;
                }

                if cmd == "else" {
                    let mut syntax_error = false;
                    match self.modes.get_mut(0) {
                        Some(mode) => mode.value = !mode.value,
                        None => syntax_error = true,
                    }
                    if syntax_error {
                        println!("Syntax error: else found with no previous if");
                    }
                    return;
                }

                if cmd == "fi" {
                    let mut syntax_error = false;
                    if self.modes.len() > 0 {
                        self.modes.remove(0);
                    } else {
                        syntax_error = true;
                    }
                    if syntax_error {
                        println!("Syntax error: fi found with no previous if");
                    }
                    return;
                }

                for mode in self.modes.iter() {
                    if !mode.value {
                        return;
                    }
                }

                //Set variables
                match cmd.find('=') {
                    Some(i) => {
                        let name = cmd[0 .. i].to_string();
                        let mut value = cmd[i + 1 .. cmd.len()].to_string();

                        if name.len() == 0 {
                            return;
                        }

                        for i in 1..args.len() {
                            match args.get(i) {
                                Some(arg) => value = value + " " + &arg,
                                None => (),
                            }
                        }

                        if value.len() == 0 {
                            let mut remove = -1;
                            for i in 0..self.variables.len() {
                                match self.variables.get(i) {
                                    Some(variable) => if variable.name == name {
                                        remove = i as isize;
                                        break;
                                    },
                                    None => break,
                                }
                            }

                            if remove >= 0 {
                                self.variables.remove(remove as usize);
                            }
                        } else {
                            for variable in self.variables.iter_mut() {
                                if variable.name == name {
                                    variable.value = value;
                                    return;
                                }
                            }

                            self.variables.push(Variable {
                                name: name,
                                value: value,
                            });
                        }
                        return;
                    }
                    None => (),
                }

                //Commands
                for command in self.commands.iter() {
                    if &command.name == cmd {
                        (*command.main)(&args);
                        return;
                    }
                }

                println!("Unknown command: '{}'", cmd);

            }
            None => (),
        }
    }

    /// Run the application
    pub fn main(&mut self) {
        console_title(&"Terminal".to_string());

        println!("Type help for a command list");
        if let Some(arg) = args().get(1) {
            let command = "run ".to_string() + arg;
            println!("# {}", command);
            self.on_command(&command);
        }

        while let Some(command) = readln!() {
            println!("# {}", command);
            if command.len() > 0 {
                self.on_command(&command);
            }
        }
    }
}

pub fn main() {
    unsafe {
        let mut app = box Application::new();
        application = app.deref_mut();
        app.main();
    }
}
