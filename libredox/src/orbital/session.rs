use alloc::arc::Arc;
use alloc::boxed::Box;
use core::{mem, slice, cmp};
use core::ops::{Deref, DerefMut};
use package::Package;
use fs::File;
use io::*;
use graphics::bmp::BmpFile;
use graphics::color::Color;
use graphics::point::Point;
use graphics::size::Size;
use graphics::display::Display;
use event::{Event, EventOption, KeyEvent, MouseEvent, DisplayEvent};
use orbital::window::*;
use vec::Vec;
use string::*;
use syscall::{sys_execve, sys_yield};
use sync::mutex::Mutex;
use session_commands::*;

use debug::debug;

#[derive(Clone)]
pub struct SessionQueue {
    pub queue: Arc<Mutex<Queue<SessionCommand>>>,
}

/// A session
pub struct Session {
    /// The display representation
    pub display: Box<Display>,
    /// The display device
    pub display_file: File,
    /// The background image
    pub background: BmpFile,
    /// The cursor icon
    pub cursor: BmpFile,
    /// The mouse point
    pub mouse_point: Point,
    last_mouse_event: MouseEvent,
    /// The packages (applications)
    pub packages: Vec<Box<Package>>,
    /// Open windows
    pub windows: Vec<*mut Window>,
    /// Ordered windows
    pub windows_ordered: Vec<*mut Window>,
    /// Redraw
    pub redraw: bool,
    /// Events `device`
    pub events: File,
    pub font: Vec<u8>,
    pub suspend_display: bool,
}

impl Session {
    pub fn commands() -> SessionQueue {
        unsafe {
            static mut COMMAND_QUEUE: *const SessionQueue = 0 as *const SessionQueue;
            static mut initialized: bool = false;
            if !initialized {
    
                let queue = SessionQueue {
                    queue: Arc::new(Mutex::new(Queue::new()))
                };
    
                COMMAND_QUEUE = mem::transmute(Box::new(queue));
                initialized = true;
            }
            (*COMMAND_QUEUE).clone()
        }
    }

    pub fn new() -> Option<Box<Self>> {
        // TODO: this font gets opened everywhere, should probably not do that
        // LazyOxen
        let mut font = Vec::new();
        if let Some(mut font_file) = File::open("file:///ui/unifont.font") {
            font_file.read_to_end(&mut font);
            if let Some(mut events) = File::open("events://") {
                if let Some(mut display_file) = File::open("display://") {
                    let mut dimensions = box Size::new(0,0);
                    let dimensions_ptr: *mut Size = dimensions.deref_mut();
                    match display_file.read(&mut unsafe {
                        slice::from_raw_parts_mut(dimensions_ptr as *mut u8, mem::size_of::<Size>())}) {
                        Some(_) => (),
                        None    => return None,
                    }

                    let ret =
                    box Session {
                            display: Display::new(dimensions.width, dimensions.height),
                            display_file: display_file,
                            background: BmpFile::load("file:///ui/background.bmp"),
                            cursor: BmpFile::load("file:///ui/cursor.bmp"),
                            mouse_point: Point::new(0, 0),
                            last_mouse_event: MouseEvent {
                                x: 0,
                                y: 0,
                                left_button: false,
                                middle_button: false,
                                right_button: false,
                                position_update: false,
                            },
                            packages: Self::load_packages(),
                            windows: Vec::new(),
                            windows_ordered: Vec::new(),
                            redraw: true,
                            events: events,
                            font: font,
                            suspend_display: false,
                    };
                    Some(ret)
                } else { // TODO: some kind of diagnostics would be nice...
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    fn load_packages() -> Vec<Box<Package>> {
        match File::open("file:///apps/") {
            Some(mut dir) => {
                let mut vec: Vec<u8> = Vec::new();
                dir.read_to_end(&mut vec);
                //TODO: get rid of unwrap
                // LazyOxen
                String::from_utf8(vec).unwrap()
                    .lines()
                    .filter(|x| x.ends_with("/"))
                    .fold(Vec::<Box<Package>>::new(),|mut packages, folder| {
                          packages.push(Package::from_path(&("file:///apps/".to_string() + folder)));
                          packages
                    })
                },
            None => Vec::<Box<Package>>::new(),
        }
    }

    pub unsafe fn add_window(&mut self, add_window_ptr: *mut Window) {
        self.windows.push(add_window_ptr);
        self.windows_ordered.push(add_window_ptr);
        self.redraw = true;
    }

    /// Remove a window
    pub unsafe fn remove_window(&mut self, remove_window_ptr: *mut Window) {
        let mut i = 0;
        while i < self.windows.len() {
            let mut remove = false;

            match self.windows.get(i) {
                Some(window_ptr) => if *window_ptr == remove_window_ptr {
                    remove = true;
                } else {
                    i += 1;
                },
                None => break,
            }

            if remove{
                self.windows.remove(i);
            }
        }

        i = 0;
        while i < self.windows_ordered.len() {
            let mut remove = false;

            match self.windows_ordered.get(i) {
                Some(window_ptr) => if *window_ptr == remove_window_ptr {
                    remove = true;
                } else {
                    i += 1;
                },
                None => break,
            }

            if remove {
                self.windows_ordered.remove(i);
            }
        }

        self.redraw = true;
    }

    fn on_key(&mut self, key_event: KeyEvent) {
        if !self.windows.is_empty() {
            match self.windows.get(self.windows.len() - 1) {
                Some(window_ptr) => {
                    unsafe {
                        (**window_ptr).on_key(key_event);
                        self.redraw = true;
                    }
                }
                None => (),
            }
        }
    }

    fn on_mouse(&mut self, mouse_event: MouseEvent) {
        let mut catcher = -1;
        let num_windows = self.windows.len();

        if mouse_event.y >= self.display.height as isize - 32 {
            if mouse_event.left_button {
                let mut x = 0;
                for package in self.packages.iter() {
                    if !package.icon.data.is_empty() {
                        if mouse_event.x >= x &&
                           mouse_event.x < x + package.icon.width() as isize {
                            unsafe { 
                                sys_execve((package.binary.to_string() + "\0").as_ptr());
                            }
                        }
                        x += package.icon.width() as isize;
                    }
                }

                let mut chars = 32;
                while chars > 4 &&
                      (x as usize + (chars * 8 + 3 * 4) * num_windows) >
                      self.display.width + 32 {
                    chars -= 1;
                }

                x += 4;
                for window_ptr in self.windows_ordered.iter() {
                    let w = (chars*8 + 2*4) as usize;
                    if mouse_event.x >= x && mouse_event.x < x + w as isize {
                        for j in 0..num_windows {
                            match self.windows.get(j) {
                                Some(catcher_window_ptr) =>
                                    if catcher_window_ptr == window_ptr {
                                    unsafe {
                                        if j == num_windows - 1 {
                                            (**window_ptr).minimized = !(**window_ptr).minimized;
                                        } else {
                                            catcher = j as isize;
                                            (**window_ptr).minimized = false;
                                        }
                                    }
                                    break;
                                },
                                None => break,
                            }
                        }
                        self.redraw = true;
                        break;
                    }
                    x += w as isize;
                }
            }
        } else {
            for reverse_i in 0..num_windows {
                let i = num_windows - 1 - reverse_i;
                match self.windows.get(i) {
                    Some(window_ptr) => unsafe {
                        if (**window_ptr).on_mouse(mouse_event, catcher < 0) {
                            catcher = i as isize;

                            self.redraw = true;
                        }
                    },
                    None => (),
                }
            }
        }

        if catcher >= 0 && catcher < num_windows as isize - 1 {
            let window_ptr = self.windows.remove(catcher as usize);
            self.windows.push(window_ptr);
        }

        self.last_mouse_event = mouse_event;
    }

    fn on_display(&mut self, display_event:DisplayEvent) {
        self.suspend_display = display_event.restricted;
        self.redraw = !self.suspend_display;
    }

    /// Redraw screen
    pub unsafe fn redraw(&mut self) {
        if self.redraw && !self.suspend_display {
            self.display.set(Color::rgb(75, 163, 253));
            if !self.background.data.is_empty() {
                self.background.draw(&self.display,
                                     Point::new((self.display.width as isize -
                                                 self.background.width() as isize) /
                                                2,
                                                (self.display.height as isize -
                                                 self.background.height() as isize) /
                                                2));
            }

            for i in 0..self.windows.len() {
                match self.windows.get(i) {
                    Some(window_ptr) => {
                        (**window_ptr).focused = i == self.windows.len() - 1;
                        (**window_ptr).draw(&self.display);
                    }
                    None => (),
                }
            }

            self.display.rect(Point::new(0, self.display.height as isize - 32),
                              Size::new(self.display.width, 32),
                              Color::rgba(0, 0, 0, 128));

            let mut x = 0;
            for package in self.packages.iter() {
                if !package.icon.data.is_empty()  {
                    let y = self.display.height as isize - package.icon.height() as isize;
                    if self.mouse_point.y >= y && self.mouse_point.x >= x &&
                       self.mouse_point.x < x + package.icon.width() as isize {
                        self.display.rect(Point::new(x, y),
                                          package.icon.size(),
                                          Color::rgba(128, 128, 128, 128));

                        self.display.rect(Point::new(x, y - 16),
                                          Size::new(package.name.len() * 8, 16),
                                          Color::rgba(0, 0, 0, 128));

                        let mut c_x = x;
                        for c in package.name.chars() {
                            self.display
                                .char(&self.font, Point::new(c_x, y - 16), c, Color::rgb(255, 255, 255));
                            c_x += 8;
                        }
                    }
                    package.icon.draw(&self.display, Point::new(x, y));
                    x += package.icon.width() as isize;
                }
            }

            let mut chars = 32;
            while chars > 4 &&
                  (x as usize + (chars * 8 + 3 * 4) * self.windows.len()) >
                  self.display.width + 32 {
                chars -= 1;
            }

            x += 4;
            for window_ptr in self.windows_ordered.iter() {
                let w = (chars*8 + 2*4) as usize;
                self.display.rect(Point::new(x, self.display.height as isize - 32),
                                  Size::new(w, 32),
                                  (**window_ptr).border_color);
                x += 4;

                for (i, c) in (0..chars).zip((**window_ptr).title().chars()) {
                    if c != '\0' {
                        self.display.char(&self.font, Point::new(x, self.display.height as isize - 24),
                                          c,
                                          (**window_ptr).title_color);
                    }
                    x += 8;
                }
                x += 8;
            }

            if !self.cursor.data.is_empty() {
                self.display.image_alpha(self.mouse_point,
                                         self.cursor.data_ptr(),
                                         self.cursor.size());
            } else {
                self.display.char(&self.font, Point::new(self.mouse_point.x - 3, self.mouse_point.y - 9),
                                  'X',
                                  Color::rgb(255, 255, 255));
            }

            self.display.flip();
            let buf: &[u8] = slice::from_raw_parts(self.display.onscreen as *const u8, self.display.size);
            self.display_file.seek(SeekFrom::Start(0));
            self.display_file.write(buf);
            self.display_file.sync();
            self.redraw = false;
        }
    }

    pub fn event(&mut self, event: Event) {
        match event.to_option() {
            EventOption::Mouse(mouse_event) => {
                if mouse_event.position_update {
                    let mouse_x = mouse_event.x as usize * self.display.width / 32768;
                    let mouse_y = mouse_event.y as usize * self.display.height / 32768;
                    self.mouse_point.x = 
                        cmp::max(0,
                                 cmp::min(self.display.width as isize -1,
                                          mouse_x as isize));
                    self.mouse_point.y = 
                        cmp::max(0,
                                 cmp::min(self.display.height as isize -1,
                                          mouse_y as isize));
                                  
                    self.redraw = true;
                    let evt_x = self.mouse_point.x;
                    let evt_y = self.mouse_point.y;
                    self.on_mouse(MouseEvent {
                        x: evt_x,
                        y: evt_y,
                        left_button: mouse_event.left_button,
                        right_button: mouse_event.right_button,
                        middle_button: mouse_event.middle_button,
                        position_update: false,
                    });
                } else {
                    self.on_mouse(mouse_event);
                }
            },
            EventOption::Key(key_event) => self.on_key(key_event),
            EventOption::Display(display_event) => self.on_display(display_event),
            _ =>  (),
        }
    }

    pub unsafe fn exec() -> ! {
        let mut session = Session::new().unwrap();
        session.redraw = true;
        session.redraw();
        loop {
            let mut event = box Event::new();
            let event_ptr: *mut Event = event.deref_mut();
            let data_read = session.events.read(&mut unsafe {
                    slice::from_raw_parts_mut(event_ptr as *mut u8, mem::size_of::<Event>())});
            match data_read {
                Some(0) => (),
                Some(_) => {
                    session.event(*event);
                } ,
                None => (),
            };

            let mut q = Session::commands();
            let mut guard = q.queue.lock();
            let mut cmds = guard.deref_mut();
            let mut i = cmds.len();
            session.redraw = session.redraw || i > 0;
            while i > 0 {
                match cmds.pop() {
                    Some(cmd) => match cmd {
                        SessionCommand::RemoveWindow(win_ptr) => session.remove_window(win_ptr),
                        SessionCommand::AddWindow(win_ptr) => session.add_window(win_ptr),
                        /*
                        SessionCommand::RemoveWindow(win_ptr) => session.remove_window(*win_ptr.borrow().deref()),
                        SessionCommand::AddWindow(win_ptr) => session.add_window(*win_ptr.borrow().deref()),
                        */
                        SessionCommand::Redraw => (), //session.redraw = true,
                    },
                    None => (),
                }
                i = i - 1;
            }
            session.redraw();
        }
    }
}
