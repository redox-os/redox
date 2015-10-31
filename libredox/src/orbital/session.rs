use alloc::boxed::Box;

use core::{mem};

use fs::File;
use io::*;
use package::Package;
use graphics::bmp::BMPFile;
use graphics::color::Color;
use graphics::point::Point;
use graphics::size::Size;
use graphics::display::Display;
use event::{Event, EventOption, KeyEvent, MouseEvent, DisplayEvent, EventData};
use orbital::window::*;
use vec::Vec;
use string::*;
use syscall::sys_execve;
use core::{slice};

pub struct Session {
    pub display: Box<Display>,
    pub display_file: File,
    pub background: BMPFile,
    pub cursor: BMPFile,
    pub mouse_point: Point,
    last_mouse_event: MouseEvent,
    pub packages: Vec<Box<Package>>,
    pub windows: Vec<*mut Window>,
    pub windows_ordered: Vec<*mut Window>,
    pub redraw: bool,
    pub events: File,
    pub font: Vec<u8>,
    pub suspend_display: bool,
}

static mut session_ptr: *mut Session = 0 as *mut Session;

impl Session {
    // TODO: make this safe
    pub unsafe fn session() -> *mut Session {
        session_ptr
    }

    fn load_packages() -> Vec<Box<Package>> {
        match File::open("file:///apps/") {
            Some(mut dir) => {
                let mut vec: Vec<u8> = Vec::new();
                dir.read_to_end(&mut vec);
                //TODO: handle this properly
                // LazyOxen
                let dir_listing = String::from_utf8(vec).unwrap();
                dir_listing.split("\n")
                    .filter(|x| x.ends_with("/"))
                    .fold(Vec::<Box<Package>>::new(),|mut packages, folder| {
                          packages.push(Package::from_path(&("file:///apps/".to_string() + folder)));
                          packages
                    })
                },
            None => Vec::<Box<Package>>::new(),
        }
    }

    fn new() -> Box<Self> {
        // TODO: this font gets opened everywhere, should probably not do that
        // LazyOxen
        let mut disp = File::open("display://").unwrap();
        let mut font = Vec::new();
        if let Some(mut font_file) = File::open("file:///ui/unifont.font") {
            font_file.read_to_end(&mut font);
            if let Some(mut events) = File::open("events://") {
                if let Some(mut display_file) = File::open("display://") {
                    let ret =
                    box Session {
                            display: Display::root(),
                            display_file: display_file,
                            background: BMPFile::load("file:///ui/background.bmp"),
                            cursor: BMPFile::load("file:///ui/cursor.bmp"),
                            mouse_point: Point::new(0, 0),
                            last_mouse_event: MouseEvent {
                                x: 0,
                                y: 0,
                                left_button: false,
                                middle_button: false,
                                right_button: false,
                            },
                            packages: Self::load_packages(),
                            windows: Vec::new(),
                            windows_ordered: Vec::new(),
                            redraw: false,//true,
                            events: events,
                            font: font,
                            suspend_display: false,
                    };
                    let colors:Vec<u32> = vec![0xFF00FF00; 640*480];
                        unsafe { 
                            let u8s = mem::transmute::<&[u32],&[u8]>(&colors[..]); 
                            disp.write(u8s);
                            disp.sync();
                    }
                    ret
                } else {
                    let colors:Vec<u32> = vec![0xFF0000FF; 640*480];
                        unsafe { 
                            let u8s = mem::transmute::<&[u32],&[u8]>(&colors[..]); 
                            disp.write(u8s);
                            disp.sync();
                    }
                    panic!("{}: unable to connect to display://", file!());
                }
            } else {
                    let colors:Vec<u32> = vec![0xFFFF0000; 640*480];
                        unsafe { 
                            let u8s = mem::transmute::<&[u32],&[u8]>(&colors[..]); 
                            disp.write(u8s);
                            disp.sync();
                    }
                panic!("{}: unable to connect to events://", file!());
            }
        } else {
                    let colors:Vec<u32> = vec![0xFFFF00FF; 640*480];
                        unsafe { 
                            let u8s = mem::transmute::<&[u32],&[u8]>(&colors[..]); 
                            disp.write(u8s);
                            disp.sync();
                    }
            panic!("{}: unable to open font file", file!());
        }
    }

    pub unsafe fn add_window(&mut self, add_window_ptr: *mut Window) {
        self.windows.push(add_window_ptr);
        self.windows_ordered.push(add_window_ptr);
        self.redraw = true;
    }

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

            if remove {
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
        if self.windows.len() > 0 {
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
        /*
        let mouse_x = (mouse_event.x * (*::session_ptr).display.width) / 32768;
        let mouse_y = (y * (*::session_ptr).display.height) / 32768;

(*::session_ptr).mouse_point.x =
 cmp::max(0,
  cmp::min((*::session_ptr).display.width as isize -
   1,
   mouse_x as isize));
(*::session_ptr).mouse_point.y =
 cmp::max(0,
  cmp::min((*::session_ptr).display.height as isize -
  1,
  mouse_y as isize));
  */

    fn on_mouse(&mut self, mouse_event: MouseEvent) {
        let mut catcher = -1;

        if mouse_event.y >= self.display.height as isize - 32 {
            if !mouse_event.left_button && self.last_mouse_event.left_button {
                let mut x = 0;
                for package in self.packages.iter() {
                    //if package.icon.data.len() > 0 {
                    // TODO: not sure this is really kosher
                    // LazyOxen
                    if package.icon.width() > 0 {
                        if mouse_event.x >= x &&
                           mouse_event.x < x + package.icon.width() as isize {
                            unsafe { sys_execve((package.binary.to_string() + "\0").as_ptr()) };
                        }
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
                    if mouse_event.x >= x && mouse_event.x < x + w as isize {
                        for j in 0..self.windows.len() {
                            match self.windows.get(j) {
                                Some(catcher_window_ptr) =>
                                    if catcher_window_ptr == window_ptr {
                                    unsafe {
                                        if j == self.windows.len() - 1 {
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
            for reverse_i in 0..self.windows.len() {
                let i = self.windows.len() - 1 - reverse_i;
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

        if catcher >= 0 && catcher < self.windows.len() as isize - 1 {
            let removed_window = self.windows.remove(catcher as usize);
            self.windows.push(removed_window);
        }

        self.last_mouse_event = mouse_event;
    }


    fn on_display(&mut self, display_event:DisplayEvent) {
        self.suspend_display = display_event.restricted;
    }

    pub unsafe fn redraw(&mut self) {
            let colors:Vec<u32> = vec![0xFFFFFFFF; 640*480];
            unsafe { 
                let u8s = mem::transmute::<&[u32],&[u8]>(&colors[..]); 
                self.display_file.write(u8s);
                self.display_file.sync();
            }
            /*
        if self.redraw && !self.suspend_display {
            self.display.set(Color::rgb(75, 163, 253));
            //LazyOxen
            if self.background.width() > 0 {
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
                //LazyOxen
                if package.icon.width() > 0 {
                    let y = self.display.height as isize - package.icon.height() as isize;
                    if self.mouse_point.y >= y && self.mouse_point.x >= x &&
                       self.mouse_point.x < x + package.icon.width() as isize {
                        self.display.rect(Point::new(x, y),
                                          package.icon.size(),
                                          Color::rgba(128, 128, 128, 128));

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

            //TODO: no hacky checks
            //LazyOxen
            if self.cursor.width() > 0 {
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
            self.display_file.write(buf);//data);
            self.display_file.sync();
            self.redraw = false;
        }
        */
    }

    pub fn event(&mut self, event: Event) {
        match event.to_option() {
            EventOption::Mouse(mouse_event) => self.on_mouse(mouse_event),
            EventOption::Key(key_event) => self.on_key(key_event),
            EventOption::Display(display_event) => self.on_display(display_event),
            _ => (),
        }
    }

    pub unsafe fn exec() -> ! {
        //session_ptr = Box::into_raw(Session::new());
        /*
        if let Some(mut display) = File::open("display://") {
            if let Some(mut events) = File::open("events://") {
                let mut data = vec![0,64];
                match events.read(&mut data) {
                    Some(_) => {
                       let colors:Vec<u32> = vec![0xFF00FF00; 640*480];
                       unsafe { 
                            let u8s = mem::transmute::<&[u32],&[u8]>(&colors[..]); 
                            display.write(u8s);
                            display.sync();
                        }
                    },
                    None => {
                       let colors:Vec<u32> = vec![0xFF00FFFF; 640*480];
                       unsafe { 
                            let u8s = mem::transmute::<&[u32],&[u8]>(&colors[..]); 
                            display.write(u8s);
                            display.sync();
                        }
                    }
                }
            }
        }
        */
        let mut ses = Session::new();
        //ses.redraw();
        loop {
            //ses.redraw();
            // TODO: maybe get rid of the event data alias? it makes this bit
            //  of code look funky
            //  LazyOxen
            /*
            let mut evt: EventData = [0;4];
            let buf = mem::transmute::<&mut [isize],&mut [u8]>(&mut evt[..]);
            (*session_ptr).events.read(buf);
            (*session_ptr).event(Event::from_data(evt));
            */
            //(*session_ptr).redraw();
        }
    }
}
