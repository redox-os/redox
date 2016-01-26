#![feature(box_syntax)]

extern crate orbital;

extern crate system;

use std::thread;

use system::error::{Error, ENOENT};
use system::scheme::{Packet, Scheme};

use orbital::Color;

use self::display::Display;
use self::image::Image;

pub mod display;
pub mod image;

struct OrbitalScheme;

impl OrbitalScheme {
    fn new() -> OrbitalScheme {
        OrbitalScheme
    }
}

impl Scheme for OrbitalScheme {}

struct Window {
    x: i32,
    y: i32,
    image: Image
}

impl Window {
    fn draw(&mut self, display: &mut Display) {
        let mut display_roi = display.image.roi(self.x, self.y, self.image.width(), self.image.height());
        display_roi.blend(&self.image.as_roi());
    }
}

fn main() {
    match Display::new() {
        Ok(mut display) => {
            println!("- Orbital: Found Display {}x{}", display.width(), display.height());
            println!("    Console: Press F1");
            println!("    Desktop: Press F2");

            let bg = Color::rgb(32, 32, 32);;

            let mut windows = Vec::new();

            windows.push(Window {
                x: 50,
                y: 50,
                image: Image::new_with_color(400, 200, Color::rgba(255, 255, 255, 128))
            });

            windows.push(Window {
                x: 100,
                y: 100,
                image: Image::new_with_color(200, 200, Color::rgba(253, 163, 75, 128))
            });

            windows.push(Window {
                x: 200,
                y: 200,
                image: Image::new_with_color(200, 200, Color::rgba(75, 163, 253, 128))
            });

            loop {
                for window in windows.iter_mut() {
                    window.draw(&mut display);
                }
                display.flip();

                thread::yield_now();
            }

            /*
            let mut scheme = OrbitalScheme::new();
            let mut socket = File::create(":orbital").unwrap();
            loop {
                let mut packet = Packet::default();
                if socket.read(&mut packet).unwrap() == 0 {
                    panic!("Unexpected EOF");
                }
                //println!("Recv {:?}", packet);

                scheme.handle(&mut packet);

                socket.write(&packet).unwrap();
                //println!("Sent {:?}", packet);
            }
            */
        },
        Err(err) => println!("- Orbital: No Display Found: {}", err)
    }
}
