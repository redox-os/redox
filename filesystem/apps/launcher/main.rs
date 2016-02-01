extern crate orbital;

use orbital::{BmpFile, Color, EventOption, Window};

use std::fs::File;
use std::io::Read;
use std::process::Command;
use std::thread;

use package::Package;

pub mod package;

fn draw(window: &mut Window, packages: &Vec<Box<Package>>, shutdown: &BmpFile, mouse_x: i32, mouse_y: i32){
    window.set(Color::rgba(0, 0, 0, 128));

    let mut x = 0;
    for package in packages.iter() {
        if package.icon.has_data() {
            let y = window.height() as isize - package.icon.height() as isize;

            if mouse_y >= y as i32 && mouse_x >= x && mouse_x < x + package.icon.width() as i32 {
                window.rect(x as i32, y as i32,
                                  package.icon.width() as u32, package.icon.height() as u32,
                                  Color::rgba(128, 128, 128, 128));

                /*
                window.rect(x as i32, y as i32 - 16,
                    package.name.len() as u32 * 8, 16,
                    Color::rgba(0, 0, 0, 128));

                let mut c_x = x;
                for c in package.name.chars() {
                    window.char(c_x as i32, y as i32 - 16,
                              c,
                              Color::rgb(255, 255, 255),
                              self.font.as_ptr() as usize);
                    c_x += 8;
                }
                */
            }

            window.image(x as i32, y as i32,
                        package.icon.width() as u32,
                        package.icon.height() as u32,
                        &package.icon);
            x = x + package.icon.width() as i32;
        }
    }

    if shutdown.has_data() {
        x = window.width() as i32 - shutdown.width() as i32;
        let y = window.height() as isize - shutdown.height() as isize;

        if mouse_y >= y as i32 && mouse_x >= x &&
           mouse_x < x + shutdown.width() as i32 {
            window.rect(x as i32, y as i32,
                              shutdown.width() as u32, shutdown.height() as u32,
                              Color::rgba(128, 128, 128, 128));
        }

        window.image(x as i32, y as i32,
                        shutdown.width() as u32, shutdown.height() as u32,
                        &shutdown);
        x = x + shutdown.width() as i32;
    }

    window.sync();
}

fn main() {
    let mut packages: Vec<Box<Package>> = Vec::new();

    //TODO: Use a directory walk
    match File::open("file:/apps/") {
        Ok(mut file) => {
            let mut string = String::new();
            if let Ok(_) = file.read_to_string(&mut string) {
                for folder in string.lines() {
                    if folder.ends_with('/') {
                        packages.push(Package::from_path(&("file:/apps/".to_string() + &folder)));
                    }
                }
            }
        }
        Err(err) => println!("Failed to open apps: {}", err),
    }
    /*
        for package in packages.iter() {
            let mut accepted = false;
            for accept in package.accepts.iter() {
                if (accept.starts_with('*') &&
                    path.ends_with(&accept.get_slice(Some(1), None))) ||
                   (accept.ends_with('*') &&
                    path.starts_with(&accept.get_slice(None, Some(accept.len() - 1)))) {
                    accepted = true;
                    break;
                }
            }
            if accepted {
                if Command::new(&package.binary).arg(&path).spawn_scheme().is_none() {
                    println!("{}: Failed to launch", package.binary);
                }
                break;
            }
        }
    */

    let shutdown = BmpFile::from_path("file:/ui/actions/system-shutdown.bmp");
    if ! shutdown.has_data() {
        println!("Failed to read shutdown icon");
    }

    let mut window = Window::new(0, 568, 800, 32, "Launcher").unwrap();

    draw(&mut window, &packages, &shutdown, -1, -1);
    'running: loop {
        while let Some(event) = window.poll() {
            match event.to_option() {
                EventOption::Mouse(mouse_event) => {
                    draw(&mut window, &packages, &shutdown, mouse_event.x, mouse_event.y);

                    if mouse_event.left_button {
                        let mut x = 0;
                        for package in packages.iter() {
                            if package.icon.has_data() {
                                if mouse_event.x >= x && mouse_event.x < x + package.icon.width() as i32 {
                                    if let Err(err) = Command::new(&package.binary).spawn() {
                                        println!("{}: Failed to launch: {}", package.binary, err);
                                    }
                                }
                                x = x + package.icon.width() as i32;
                            }
                        }

                        if shutdown.has_data() {
                            x = window.width() as i32 - shutdown.width() as i32;
                            let y = window.height() as i32 - shutdown.height() as i32;
                            if mouse_event.y >= y && mouse_event.x >= x &&
                               mouse_event.x < x + shutdown.width() as i32 {
                                   File::create("acpi:off");
                            }
                        }
                    }
                },
                EventOption::Quit(_) => break 'running,
                _ => ()
            }
        }

        thread::yield_now();
    }
}
