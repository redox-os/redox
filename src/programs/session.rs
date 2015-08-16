use core::char;
use core::cmp::max;
use core::cmp::min;
use core::option::Option;

use common::debug::*;

use graphics::color::*;
use graphics::size::*;

use programs::common::*;
use programs::editor::*;
use programs::executor::*;
use programs::viewer::*;

pub struct Session {
    pub display: Display,
    pub mouse_point: Point,
    pub items: Vec<Rc<SessionItem>>,
    pub current_item: isize,
    pub modules: Vec<Rc<SessionModule>>,
    pub events: Vec<URL>,
    pub redraw: usize
}

impl Session {
    pub fn new() -> Session {
        Session {
            display: Display::new(),
            mouse_point: Point::new(0, 0),
            items: Vec::new(),
            current_item: -1,
            modules: Vec::new(),
            events: Vec::new(),
            redraw: REDRAW_ALL
        }
    }

    pub fn on_irq(&mut self, irq: u8){
        for module in self.modules.iter() {
            unsafe{
                Rc::unsafe_get_mut(module).on_irq(&mut self.events, irq);
            }
        }
    }

    pub fn on_poll(&mut self){
        for module in self.modules.iter() {
            unsafe{
                Rc::unsafe_get_mut(module).on_poll(&mut self.events);
            }
        }
    }

    pub fn open(&self, url: &URL) -> Box<Resource>{
        if url.scheme.len() == 0 {
            let mut list = String::new();

            for module in self.modules.iter() {
                let scheme = module.scheme();
                if scheme.len() > 0 {
                    if list.len() > 0 {
                        list = list + "\n" + scheme;
                    }else{
                        list = scheme;
                    }
                }
            }

            return box VecResource::new(ResourceType::Dir, list.to_utf8());
        }else{
            for module in self.modules.iter() {
                if module.scheme() == url.scheme {
                    unsafe{
                        return Rc::unsafe_get_mut(module).open(url);
                    }
                }
            }
            return box NoneResource;
        }
    }

    pub fn open_async(&self, url: &URL, callback: Box<FnBox(Box<Resource>)>){
        if url.scheme.len() == 0 {
            let mut list = String::new();

            for module in self.modules.iter() {
                let scheme = module.scheme();
                if scheme.len() > 0 {
                    if list.len() > 0 {
                        list = list + "\n" + scheme;
                    }else{
                        list = scheme;
                    }
                }
            }

            callback(box VecResource::new(ResourceType::Dir, list.to_utf8()));
        }else{
            for module in self.modules.iter() {
                if module.scheme() == url.scheme {
                    unsafe{
                        Rc::unsafe_get_mut(module).open_async(url, callback);
                    }
                    return;
                }
            }
            callback(box NoneResource);
        }
    }

    pub fn on_key(&mut self, key_event: KeyEvent){
        self.current_item = 0;
        match self.items.get(self.current_item as usize){
            Option::Some(item) => {
                unsafe {
                    Rc::unsafe_get_mut(item).on_key(&mut self.events, key_event);
                }

                self.redraw = max(self.redraw, REDRAW_ALL);
            },
            Option::None => ()
        }
        self.current_item = -1;
    }

    pub fn on_mouse(&mut self, mouse_event: MouseEvent){
        self.mouse_point.x = max(0, min(self.display.width as isize - 1, self.mouse_point.x + mouse_event.x));
        self.mouse_point.y = max(0, min(self.display.height as isize - 1, self.mouse_point.y + mouse_event.y));

        self.redraw = max(self.redraw, REDRAW_CURSOR);

        let mut catcher = 0;
        let mut allow_catch = true;
        for i in 0..self.items.len() {
            self.current_item = i as isize;
            match self.items.get(self.current_item as usize){
                Option::Some(item) => {
                    unsafe {
                        if Rc::unsafe_get_mut(item).on_mouse(&mut self.events, self.mouse_point, mouse_event, allow_catch) {
                            allow_catch = false;
                            catcher = i;

                            self.redraw = max(self.redraw, REDRAW_ALL);
                        }
                    }
                },
                Option::None => ()
            }
        }
        self.current_item = -1;

        if catcher > 0 && catcher < self.items.len() {
            match self.items.remove(catcher){
                Option::Some(item) => {
                    self.items.insert(0, item);
                },
                Option::None => ()
            }
        }
    }

    pub fn redraw(&mut self){
        if self.redraw > REDRAW_NONE {
            if self.redraw >= REDRAW_ALL {
                self.display.background();

                self.display.rect(Point::new(0, 0), Size::new(self.display.width, 18), Color::new(0, 0, 0));
                self.display.text(Point::new(self.display.width as isize/ 2 - 3*8, 1), &String::from_str("Redox"), Color::new(255, 255, 255));

                let mut erase_i: Vec<usize> = Vec::new();
                for reverse_i in 0..self.items.len() {
                    self.current_item = (self.items.len() - 1 - reverse_i) as isize;
                    match self.items.get(self.current_item as usize) {
                        Option::Some(item) => {
                            unsafe {
                                if ! Rc::unsafe_get_mut(item).draw(&self.display, &mut self.events) {
                                    erase_i.push(self.current_item as usize);
                                }
                            }
                        },
                        Option::None => ()
                    }
                }
                self.current_item = -1;

                for i in erase_i.iter() {
                    drop(self.items.remove(*i));
                }
            }

            self.display.flip();

            self.display.cursor(self.mouse_point);

            self.redraw = REDRAW_NONE;
        }
    }

    pub fn handle_events(&mut self){
        while self.events.len() > 0 {
            match self.events.remove(0){
                Option::Some(event) => {
                    if event.scheme == "r".to_string() {
                        match event.path.get(0) {
                            Option::Some(part) => self.redraw = max(self.redraw, part.to_num()),
                            Option::None => ()
                        }
                    }else if event.scheme == "m".to_string() {
                        let mut mouse_event = MouseEvent {
                            x: 0,
                            y: 0,
                            left_button: false,
                            middle_button: false,
                            right_button: false,
                            valid: true
                        };

                        match event.path.get(0) {
                            Option::Some(part) => mouse_event.x = part.to_num_signed(),
                            Option::None => ()
                        }

                        match event.path.get(1) {
                            Option::Some(part) => mouse_event.y = part.to_num_signed(),
                            Option::None => ()
                        }

                        match event.path.get(2) {
                            Option::Some(part) => mouse_event.left_button = part.to_num() > 0,
                            Option::None => ()
                        }

                        match event.path.get(3) {
                            Option::Some(part) => mouse_event.middle_button = part.to_num() > 0,
                            Option::None => ()
                        }

                        match event.path.get(4) {
                            Option::Some(part) => mouse_event.right_button = part.to_num() > 0,
                            Option::None => ()
                        }

                        self.on_mouse(mouse_event);
                    }else if event.scheme == "k".to_string() {
                        let mut key_event = KeyEvent {
                            character: '\0',
                            scancode: 0,
                            pressed: false
                        };

                        match event.path.get(0) {
                            Option::Some(part) => match char::from_u32(part.to_num() as u32){
                                Option::Some(character) => key_event.character = character,
                                Option::None => ()
                            },
                            Option::None => ()
                        }

                        match event.path.get(1) {
                            Option::Some(part) => key_event.scancode = part.to_num() as u8,
                            Option::None => ()
                        }

                        match event.path.get(2) {
                            Option::Some(part) => key_event.pressed = part.to_num() > 0,
                            Option::None => ()
                        }

                        self.on_key(key_event);
                    }else if event.scheme == "open".to_string() {
                        self.redraw = max(self.redraw, REDRAW_ALL);

                        let url_string = event.path_string();
                        let url = URL::from_string(url_string.clone());

                        let mut found = false;
                        if url_string.ends_with(".md".to_string()) || url_string.ends_with(".rs".to_string()){
                            self.items.insert(0, Rc::new(Editor::new()));
                            found = true;
                        }else if url_string.ends_with(".bin".to_string()){
                            self.items.insert(0, Rc::new(Executor::new()));
                            found = true;
                        }else if url_string.ends_with(".bmp".to_string()){
                            self.items.insert(0, Rc::new(Viewer::new()));
                            found = true;
                        }else{
                            d("No program found: ");
                            url.d();
                            dl();
                        }

                        if found {
                            self.current_item = 0;
                            match self.items.get(0) {
                                Option::Some(item) => unsafe{
                                    Rc::unsafe_get_mut(&item).load(&url);
                                },
                                Option::None => ()
                            }
                            self.current_item = -1;
                        }
                    }
                },
                Option::None => ()
            }
        }
    }
}
