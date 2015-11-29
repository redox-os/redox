use std::get_slice::GetSlice;
use std::ops::DerefMut;
use std::string::*;
use std::vec::Vec;
use std::boxed::Box;
use std::fs::*;
use std::io::*;
use std::env::*;
use std::time::Duration;
use std::to_num::*;
use std::hashmap::HashMap;
use std::process;

macro_rules! readln {
    () => ({
        let mut buffer = String::new();
        match std::io::stdin().read_to_string(&mut buffer) {
            Some(_) => Some(buffer),
            None => None
        }
    });
}

/* Magic { */
static mut application: *mut Application<'static> = 0 as *mut Application;
/* } Magic */

/// Structure which represents a Terminal's command.
/// This command structure contains a name, and the code which run the functionnality associated to this one, with zero, one or several argument(s).
/// # Example
/// ```
/// let my_command = Command {
///     name: "my_command",
///     main: box|args: &Vec<String>| {
///         println!("Say 'hello' to my command! :-D");
///     }
/// }
/// ```
pub struct Command<'a> {
    pub name: &'a str,
    pub help: &'a str,
    pub main: Box<Fn(&Vec<String>)>,
}

impl<'a> Command<'a> {
    /// Return the vector of the commands
    // TODO: Use a more efficient collection instead
    pub fn vec() -> Vec<Self> {
        let mut commands: Vec<Self> = Vec::new();

        commands.push(Command {
            name: "cat",
            help: "To display a file in the output\n    cat <your_file>",
            main: Box::new(|args: &Vec<String>| {
                let path = {
                    match args.get(1) {
                        Some(arg) => arg.clone(),
                        None => String::new(),
                    }
                };

                if let Some(mut file) = File::open(&path) {
                    let mut string = String::new();
                    match file.read_to_string(&mut string) {
                        Some(_) => println!("{}", string),
                        None => println!("Failed to read: {}", path),
                    }
                } else {
                    println!("Failed to open file: {}", path);
                }
            }),
        });

        commands.push(Command {
            name: "cd",
            help: "To change the current directory\n    cd <your_destination>",
            main: Box::new(|args: &Vec<String>| {
                match args.get(1) {
                    Some(path) => {
                        if !change_cwd(&path) {
                            println!("Bad path: {}", path);
                        }
                    }
                    None => println!("No path given")
                }
            }),
        });

        commands.push(Command {
            name: "echo",
            help: "To display some text in the output\n    echo Hello world!",
            main: Box::new(|args: &Vec<String>| {
                let echo = args.iter()
                    .skip(1)
                    .fold(String::new(), |string, arg| string + " " + arg);
                println!("{}", echo.trim());
            }),
        });

        commands.push(Command {
            name: "else",
            help: "",
            main: Box::new(|_: &Vec<String>| {}),
        });

        commands.push(Command {
            name: "exec",
            help: "To execute a binary in the output\n    exec <my_binary>",
            main: Box::new(|args: &Vec<String>| {
                if let Some(path) = args.get(1) {
                    let mut command = process::Command::new(path);
                    for arg in args.get_slice(Some(2), None) {
                        command.arg(arg);
                    }

                    if let Some(mut child) = command.spawn() {
                        if let Some(status) = child.wait() {
                            if let Some(code) = status.code() {
                                unsafe { (*application).set_var("?", &format!("{}", code)) };
                            } else {
                                println!("{}: No child exit code", path);
                            }
                        } else {
                            println!("{}: Failed to wait", path);
                        }
                    } else {
                        println!("{}: Failed to execute", path);
                    }
                }
            }),
        });

        commands.push(Command {
            name: "exit",
            help: "To exit the curent session",
            main: Box::new(|_: &Vec<String>| {}),
        });

        commands.push(Command {
            name: "fi",
            help: "",
            main: Box::new(|_: &Vec<String>| {}),
        });

        commands.push(Command {
            name: "if",
            help: "",
            main: Box::new(|_: &Vec<String>| {}),
        });

        commands.push(Command {
            name: "ls",
            help: "To list the content of the current directory\n    ls",
            main: Box::new(|args: &Vec<String>| {
                let path = {
                    match args.get(1) {
                        Some(arg) => arg.clone(),
                        None => String::new(),
                    }
                };

                if let Some(dir) = read_dir(&path) {
                    for entry in dir {
                        println!("{}", entry.path());
                    }
                } else {
                    println!("Failed to open directory: {}", path);
                }
            }),
        });

        commands.push(Command {
            name: "mkdir",
            help: "To create a directory in the current directory\n    mkdir <my_new_directory>",
            main: Box::new(|args: &Vec<String>| {
                match args.get(1) {
                    Some(dir_name) => if DirEntry::create(dir_name).is_none() {
                        println!("Failed to create {}", dir_name);
                    },
                    None => println!("No name provided")
                }
            }),
        });

        commands.push(Command {
            name: "pwd",
            help: "To output the path of the current directory\n    pwd",
            main: Box::new(|_: &Vec<String>| {
                if let Some(file) = File::open("") {
                    if let Some(path) = file.path() {
                        println!("{}", path);
                    } else {
                        println!("Could not get the path");
                    }
                } else {
                    println!("Could not open the working directory");
                }
            }),
        });

        commands.push(Command {
            name: "read",
            help: "To read some variables\n    read <my_variable>",
            main: Box::new(|_: &Vec<String>| {}),
        });

        commands.push(Command {
            name: "rm",
            help: "To remove a file, in the current directory\n    rm <my_file>",
            main: Box::new(|args: &Vec<String>| {
                match args.get(1) {
                    Some(file_name) => if ! unlink(file_name) {
                        println!("Failed to remove: {}", file_name);
                    },
                    None => println!("No name provided")
                }
            }),
        });

        commands.push(Command {
            name: "run",
            help: "Reads and runs a script file\n    run <my_script>",
            main: Box::new(|args: &Vec<String>| {
                if let Some(path) = args.get(1) {

                    let mut commands = String::new();
                    if let Some(mut file) = File::open(path) {
                        file.read_to_string(&mut commands);
                    }

                    for command in commands.split('\n') {
                        unsafe {
                            (*application).on_command(&command);
                        }
                    }
                }
            }),
        });

        commands.push(Command {
            name: "sleep",
            help: "Make a sleep in the current session\n    sleep <number_of_seconds>",
            main: Box::new(|args: &Vec<String>| {
                let secs = {
                    match args.get(1) {
                        Some(arg) => arg.to_num() as i64,
                        None => 0,
                    }
                };

                let nanos = {
                    match args.get(2) {
                        Some(arg) => arg.to_num() as i32,
                        None => 0,
                    }
                };

                println!("Sleep: {} {}", secs, nanos);
                let remaining = Duration::new(secs, nanos).sleep();
                println!("Remaining: {} {}", remaining.secs, remaining.nanos);
            }),
        });

        commands.push(Command {
            name: "send",
            help: "To send data, via an URL\n    send <url> <data>",
            main: Box::new(|args: &Vec<String>| {
                if args.len() < 3 {
                    println!("Error: incorrect arguments");
                    println!("Usage: send <url> <data>");
                    return;
                }

                let path = {
                    match args.get(1) {
                        Some(arg) => arg.clone(),
                        None => String::new(),
                    }
                };

                if let Some(mut file) = File::open(&path) {
                    println!("URL: {:?}", file.path());

                    let string: String = args.iter()
                        .skip(2)
                        .fold(String::new(), |s, arg| s + " " + arg)
                        + "\r\n\r\n";

                    match file.write(string.trim_left().as_bytes()) {
                        Some(size) => println!("Wrote {} bytes", size),
                        None => println!("Failed to write"),
                    }

                    let mut string = String::new();
                    match file.read_to_string(&mut string) {
                        Some(_) => println!("{}", string),
                        None => println!("Failed to read"),
                    }
                }
            }),
        });

        // Simple command to create a file, in the current directory
        // The file has got the name given as the first argument of the command
        // If the command have no arguments, the command don't create the file
        commands.push(Command {
            name: "touch",
            help: "To create a file, in the current directory\n    touch <my_file>",
            main: Box::new(|args: &Vec<String>| {
                match args.get(1) {
                    Some(file_name) => if File::create(file_name).is_none() {
                        println!("Failed to create: {}", file_name);
                    },
                    None => println!("No name provided")
                }
            }),
        });

        commands.push(Command {
            name: "url_hex",
            help: "",
            main: Box::new(|args: &Vec<String>| {
                let path = {
                    match args.get(1) {
                        Some(arg) => arg.clone(),
                        None => String::new(),
                    }
                };

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
            }),
        });

        commands.push(Command {
            name: "wget",
            help: "To make some requests at a given host, using TCP protocol\n    wget <host> <request>",
            main: Box::new(|args: &Vec<String>| {
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
            }),
        });

        let mut command_helper : HashMap<String, String> = HashMap::new();

        for c in commands.iter() {
            command_helper.insert(c.name.clone().to_string(), c.help.clone().to_string());
        }

        commands.push(Command {
            name: "man",
            help: "Display a little helper for a given command\n    man ls",
            main: Box::new(move |args: &Vec<String>| {
                if let Some(command) = args.get(1) {
                    if command_helper.contains_key(&command) {
                        match command_helper.get(&command) {
                            Some(help) => println!("{}", help),
                            None => println!("Command helper not found [run 'help']...")
                        }
                    }
                    else {
                        println!("Command helper not found [run 'help']...");
                    }
                }
                else {
                    println!("Please to specify a command!");
                }
            }),
        });

        let command_list = commands.iter().fold(String::new(), |l , c| l + " " + c.name);

        commands.push(Command {
            name: "help",
            help: "Print current commands to call",
            main: Box::new(move |_: &Vec<String>| {
                println!("Commands:{}", command_list);
            }),
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
pub struct Application<'a> {
    commands: Vec<Command<'a>>,
    variables: Vec<Variable>,
    modes: Vec<Mode>,
}

impl<'a> Application<'a> {
    /// Create a new empty application
    pub fn new() -> Self {
        return Application {
            commands: Command::vec(),
            variables: Vec::new(),
            modes: Vec::new(),
        };
    }

    fn on_command(&mut self, command_string: &str) {
        //Comment
        if command_string.starts_with('#') {
            return;
        }

        //Show variables
        if command_string == "$" {
            for variable in self.variables.iter() {
                println!("{}={}", variable.name, variable.value);
            }
            return;
        }

        //Explode into arguments, replace variables
        let mut args: Vec<String> = Vec::<String>::new();
        for arg in command_string.split(' ') {
            if !arg.is_empty() {
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
        if let Some(cmd) = args.get(0) {
            if cmd == "if" {
                let mut value = false;

                if let Some(left) = args.get(1) {
                    if let Some(cmp) = args.get(2) {
                        if let Some(right) = args.get(3) {
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
                        } else {
                            println!("No right hand side");
                        }
                    } else {
                        println!("No comparison operator");
                    }
                } else {
                    println!("No left hand side");
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
                if !self.modes.is_empty() {
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

            if cmd == "read" {
                for i in 1..args.len() {
                    if let Some(arg_original) = args.get(i) {
                        let arg = arg_original.trim();
                        print!("{}=", arg);
                        if let Some(value_original) = readln!() {
                            let value = value_original.trim();
                            self.set_var(arg, value);
                        }
                    }
                }
            }

            //Set variables
            if let Some(i) = cmd.find('=') {
                let name = cmd[0 .. i].trim();
                let mut value = cmd[i + 1 .. cmd.len()].trim().to_string();

                for i in 1..args.len() {
                    if let Some(arg) = args.get(i) {
                        value = value + " " + &arg;
                    }
                }

                self.set_var(name, &value);
                return;
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
    }


    pub fn set_var(&mut self, name: &str, value: &str){
        if name.is_empty() {
            return;
        }

        if value.is_empty() {
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
                    variable.value = value.to_string();
                    return;
                }
            }

            self.variables.push(Variable {
                name: name.to_string(),
                value: value.to_string(),
            });
        }
    }

    /// Method to return the current directory
    /// If the current directory cannot be found, a default string ("?") will be returned
    pub fn get_current_directory(&mut self) -> String {
        // Return the current path
        File::open("")
            .and_then(|file| file.path())
            .unwrap_or("?".to_string())
    }

    /// Run the application
    pub fn main(&mut self) {
        println!("Type help for a command list");
        if let Some(arg) = args().get(1) {
            let command = "run ".to_string() + arg;
            println!("user@redox:{}# {}", self.get_current_directory(), command);
            self.on_command(&command);
        }

        loop {
            for mode in self.modes.iter().rev() {
                if mode.value {
                    print!("+ ");
                } else {
                    print!("- ");
                }
            }
            print!("user@redox:{}# ", self.get_current_directory());
            if let Some(command_original) = readln!() {
                let command = command_original.trim();
                if command == "exit" {
                    break;
                } else if !command.is_empty() {
                    self.on_command(&command);
                }
            } else {
                break;
            }
        }
    }
}

#[no_mangle]
pub fn main() {
    unsafe {
        let mut app = Box::new(Application::new());
        application = app.deref_mut();
        app.main();
    }
}
