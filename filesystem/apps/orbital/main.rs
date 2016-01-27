extern crate core;
extern crate system;

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Read, Write};

use system::error::{Error, Result, ENOENT, EBADF};
use system::scheme::{Packet, Scheme};

pub use self::color::Color;
pub use self::display::Display;
pub use self::event::{Event, EventOption};
pub use self::image::{Image, ImageRoi};
pub use self::window::Window;

use self::event::EVENT_KEY;

pub mod color;
pub mod display;
#[path="../../../kernel/common/event.rs"]
pub mod event;
pub mod image;
pub mod window;

struct OrbitalScheme {
    next_id: isize,
    order: Vec<usize>,
    windows: BTreeMap<usize, Window>,
}

impl OrbitalScheme {
    fn new() -> OrbitalScheme {
        OrbitalScheme {
            next_id: 1,
            order: Vec::new(),
            windows: BTreeMap::new()
        }
    }

    fn update(&mut self, display: &mut Display) {
        while let Some(event) = display.poll() {
            if event.code == EVENT_KEY {
                if let Some(id) = self.order.last() {
                    if let Some(mut window) = self.windows.get_mut(&id) {
                        window.event(event);
                    }
                }
            }
        }

        display.as_roi().set(Color::rgb(75, 163, 253));
        for id in self.order.iter() {
            if let Some(mut window) = self.windows.get_mut(&id) {
                window.draw(display);
            }
        }
        display.flip();
    }
}

impl Scheme for OrbitalScheme {
    #[allow(unused_variables)]
    fn open(&mut self, path: &str, flags: usize, mode: usize) -> Result {
        let res = path.split(":").nth(1).unwrap_or("");
        let x = res.split("/").nth(0).unwrap_or("").parse::<i32>().unwrap_or(0);
        let y = res.split("/").nth(1).unwrap_or("").parse::<i32>().unwrap_or(0);
        let width = res.split("/").nth(2).unwrap_or("").parse::<i32>().unwrap_or(0);
        let height = res.split("/").nth(3).unwrap_or("").parse::<i32>().unwrap_or(0);

        let id = self.next_id as usize;
        self.next_id += 1;
        if self.next_id < 0 {
            self.next_id = 1;
        }

        self.order.push(id);
        self.windows.insert(id, Window::new(x, y, width, height));

        Ok(id)
    }

    fn read(&mut self, id: usize, buf: &mut [u8]) -> Result {
        if let Some(mut window) = self.windows.get_mut(&id) {
            window.read(buf)
        } else {
            Err(Error::new(EBADF))
        }
    }

    fn write(&mut self, id: usize, buf: &[u8]) -> Result {
        if let Some(mut window) = self.windows.get_mut(&id) {
            window.write(buf)
        } else {
            Err(Error::new(EBADF))
        }
    }

    fn close(&mut self, id: usize) -> Result {
        let mut i = 0;
        while i < self.order.len() {
            let mut remove = false;
            if let Some(key) = self.order.get(i) {
                if *key == id {
                    remove = true;
                }
            }

            if remove {
                self.order.remove(i);
            } else {
                i += 1;
            }
        }

        if let Some(window) = self.windows.remove(&id) {
            Ok(0)
        } else {
            Err(Error::new(EBADF))
        }
    }
}

fn main() {
    let mut socket = File::create(":orbital").unwrap();
    let mut scheme = OrbitalScheme::new();
    match Display::new() {
        Ok(mut display) => {
            println!("- Orbital: Found Display {}x{}", display.width(), display.height());
            println!("    Console: Press F1");
            println!("    Desktop: Press F2");
            loop {
                let mut packet = Packet::default();
                if socket.read(&mut packet).unwrap() == 0 {
                    panic!("Unexpected EOF");
                }

                scheme.handle(&mut packet);

                scheme.update(&mut display);

                socket.write(&packet).unwrap();
            }
        },
        Err(err) => println!("- Orbital: No Display Found: {}", err)
    }
}
