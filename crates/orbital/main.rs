extern crate core;
extern crate system;

use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;

use system::error::{Error, Result, EBADF};
use system::scheme::{Packet, Scheme};

pub use self::color::Color;
pub use self::event::{Event, EventOption};
pub use self::font::Font;
pub use self::image::{Image, ImageRoi};
pub use self::socket::Socket;
pub use self::window::Window;

use self::bmp::BmpFile;
use self::event::{EVENT_KEY, EVENT_MOUSE, QuitEvent};

pub mod bmp;
pub mod color;
#[path="../../kernel/common/event.rs"]
pub mod event;
pub mod font;
pub mod image;
pub mod socket;
pub mod window;

struct OrbitalScheme {
    image: Image,
    background: Image,
    cursor: Image,
    cursor_x: i32,
    cursor_y: i32,
    dragging: bool,
    drag_x: i32,
    drag_y: i32,
    next_id: isize,
    next_x: i32,
    next_y: i32,
    order: VecDeque<usize>,
    windows: BTreeMap<usize, Window>,
    redraw: bool,
}

impl OrbitalScheme {
    fn new(width: i32, height: i32) -> OrbitalScheme {
        OrbitalScheme {
            image: Image::new(width, height),
            background: BmpFile::from_path("/ui/background.bmp"),
            cursor: BmpFile::from_path("/ui/cursor.bmp"),
            cursor_x: 0,
            cursor_y: 0,
            dragging: false,
            drag_x: 0,
            drag_y: 0,
            next_id: 1,
            next_x: 20,
            next_y: 20,
            order: VecDeque::new(),
            windows: BTreeMap::new(),
            redraw: true,
        }
    }

    fn redraw(&mut self, display: &Socket){
        if self.redraw {
            println!("Redraw {}", self.windows.len());
            self.redraw = false;
            self.image.as_roi().set(Color::rgb(75, 163, 253));
            self.image.as_roi().blend(&self.background.as_roi());

            let mut i = self.order.len();
            for id in self.order.iter().rev() {
                i -= 1;
                if let Some(mut window) = self.windows.get_mut(&id) {
                    window.draw(&mut self.image, i == 0);
                }
            }

            self.image.roi(self.cursor_x, self.cursor_y, self.cursor.width(), self.cursor.height()).blend(&self.cursor.as_roi());
            display.send_type(self.image.data()).unwrap();
        }
    }

    fn event(&mut self, event: Event){
        if event.code == EVENT_KEY {
            if let Some(id) = self.order.front() {
                if let Some(mut window) = self.windows.get_mut(&id) {
                    window.event(event);
                }
            }
        } else if event.code == EVENT_MOUSE {
            self.cursor_x = event.a as i32;
            self.cursor_y = event.b as i32;
            self.redraw = true;

            if self.dragging {
                if event.c > 0 {
                    if let Some(id) = self.order.front() {
                        if let Some(mut window) = self.windows.get_mut(&id) {
                            window.x += self.cursor_x - self.drag_x;
                            window.y += self.cursor_y - self.drag_y;
                            self.drag_x = self.cursor_x;
                            self.drag_y = self.cursor_y;
                        } else {
                            self.dragging = false;
                        }
                    } else {
                        self.dragging = false;
                    }
                } else {
                    self.dragging = false;
                }
            } else {
                let mut focus = 0;
                let mut i = 0;
                for id in self.order.iter() {
                    if let Some(mut window) = self.windows.get_mut(&id) {
                        if window.contains(event.a as i32, event.b as i32) {
                            let mut window_event = event;
                            window_event.a -= window.x as i64;
                            window_event.b -= window.y as i64;
                            window.event(window_event);
                            if event.c > 0 {
                                focus = i;
                            }
                            break;
                        } else if window.title_contains(event.a as i32, event.b as i32) {
                            if event.c > 0 {
                                focus = i;
                                if window.exit_contains(event.a as i32, event.b as i32) {
                                    window.event(QuitEvent.to_event());
                                } else {
                                    self.dragging = true;
                                    self.drag_x = self.cursor_x;
                                    self.drag_y = self.cursor_y;
                                }
                            }
                            break;
                        }
                    }
                    i += 1;
                }
                if focus > 0 {
                    if let Some(id) = self.order.remove(focus) {
                        self.order.push_front(id);
                    }
                }
            }
        }
    }
}

impl Scheme for OrbitalScheme {
    fn open(&mut self, path: &str, _flags: usize, _mode: usize) -> Result<usize> {
        println!("Open {}", path);

        let mut parts = path.split("/").skip(1);

        let mut x = parts.next().unwrap_or("").parse::<i32>().unwrap_or(0);
        let mut y = parts.next().unwrap_or("").parse::<i32>().unwrap_or(0);
        let width = parts.next().unwrap_or("").parse::<i32>().unwrap_or(0);
        let height = parts.next().unwrap_or("").parse::<i32>().unwrap_or(0);

        let mut title = parts.next().unwrap_or("").to_string();
        for part in parts {
            title.push('/');
            title.push_str(part);
        }

        let id = self.next_id as usize;
        self.next_id += 1;
        if self.next_id < 0 {
            self.next_id = 1;
        }

        if x < 0 && y < 0 {
            x = self.next_x;
            y = self.next_y;

            self.next_x += 20;
            if self.next_x + 20 >= self.image.width() {
                self.next_x = 20;
            }
            self.next_y += 20;
            if self.next_y + 20 >= self.image.height() {
                self.next_y = 20;
            }
        }

        self.order.push_front(id);
        self.windows.insert(id, Window::new(x, y, width, height, title));
        self.redraw = true;

        Ok(id)
    }

    fn read(&mut self, id: usize, buf: &mut [u8]) -> Result<usize> {
        if let Some(mut window) = self.windows.get_mut(&id) {
            window.read(buf)
        } else {
            Err(Error::new(EBADF))
        }
    }

    fn write(&mut self, id: usize, buf: &[u8]) -> Result<usize> {
        if let Some(mut window) = self.windows.get_mut(&id) {
            self.redraw = true;
            window.write(buf)
        } else {
            Err(Error::new(EBADF))
        }
    }

    fn fpath(&self, id: usize, buf: &mut [u8]) -> Result<usize> {
        if let Some(window) = self.windows.get(&id) {
            window.path(buf)
        } else {
            Err(Error::new(EBADF))
        }
    }

    fn close(&mut self, id: usize) -> Result<usize> {
        println!("Close {}", id);

        self.order.retain(|&e| e != id);

        if self.windows.remove(&id).is_some() {
            self.redraw = true;
            Ok(0)
        } else {
            Err(Error::new(EBADF))
        }
    }
}

fn event_loop(scheme_mutex: Arc<Mutex<OrbitalScheme>>, display: Socket){
    loop {
        {
            let mut scheme = scheme_mutex.lock().unwrap();
            scheme.redraw(&display);
        }

        let mut events = [Event::new(); 128];
        let count = display.receive_type(&mut events).unwrap();
        {
            let mut scheme = scheme_mutex.lock().unwrap();
            for &event in events[.. count].iter() {
                scheme.event(event);
            }
        }
        println!("Events: {}", count);
    }
}

fn server_loop(scheme_mutex: Arc<Mutex<OrbitalScheme>>, display: Socket, socket: Socket){
    loop {
        {
            let mut scheme = scheme_mutex.lock().unwrap();
            scheme.redraw(&display);
        }

        let mut packets = [Packet::default(); 128];
        let count = socket.receive_type(&mut packets).unwrap();
        /*
        {
            let mut scheme = scheme_mutex.lock().unwrap();
            for mut packet in packets[.. count].iter_mut() {
                scheme.handle(packet);
            }
        }
        socket.send_type(&packets[.. count]).unwrap();
        */
        println!("Packets: {}", count);
    }
}

fn main() {
    let socket = Socket::create(":orbital").unwrap();

    match Socket::open("display:") {
        Ok(display) => {
            let path = display.path().unwrap().to_string();
            let res = path.split(":").nth(1).unwrap_or("");
            let width = res.split("/").nth(0).unwrap_or("").parse::<i32>().unwrap_or(0);
            let height = res.split("/").nth(1).unwrap_or("").parse::<i32>().unwrap_or(0);

            println!("- Orbital: Found Display {}x{}", width, height);
            println!("    Console: Press F1");
            println!("    Desktop: Press F2");

            let scheme = Arc::new(Mutex::new(OrbitalScheme::new(width, height)));

            let display_event = display.dup().unwrap();
            let scheme_event = scheme.clone();
            thread::spawn(move || {
                event_loop(scheme_event, display_event);
            });

            server_loop(scheme, display, socket);
        },
        Err(err) => println!("- Orbital: No Display Found: {}", err)
    }
}
