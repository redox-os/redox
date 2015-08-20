use core::cmp::max;
use core::cmp::min;

use graphics::color::*;
use graphics::size::*;

use programs::common::*;
use programs::editor::*;
use programs::executor::*;
use programs::viewer::*;

pub struct Session {
    pub display: Display,
    pub mouse_point: Point,
    pub items: Vec<Box<SessionItem>>,
    pub modules: Vec<Box<SessionModule>>,
    pub redraw: usize
}

impl Session {
    pub fn new() -> Session {
        Session {
            display: Display::new(),
            mouse_point: Point::new(0, 0),
            items: Vec::new(),
            modules: Vec::new(),
            redraw: REDRAW_ALL
        }
    }

    pub fn on_irq(&mut self, irq: u8){
        for module in self.modules.iter() {
            unsafe{
                module.on_irq(irq);
            }
        }
    }

    pub fn on_poll(&mut self){
        for module in self.modules.iter() {
            unsafe{
                module.on_poll();
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
                        return module.open(url);
                    }
                }
            }
            return box NoneResource;
        }
    }

    pub fn on_key(&mut self, key_event: KeyEvent){
        match self.items.get(0){
            Option::Some(item) => {
                unsafe {
                    item.on_key(key_event);
                }

                self.redraw = max(self.redraw, REDRAW_ALL);
            },
            Option::None => ()
        }
    }

    pub fn on_mouse(&mut self, mouse_event: MouseEvent){
        self.mouse_point.x = max(0, min(self.display.width as isize - 1, self.mouse_point.x + mouse_event.x));
        self.mouse_point.y = max(0, min(self.display.height as isize - 1, self.mouse_point.y + mouse_event.y));

        self.redraw = max(self.redraw, REDRAW_CURSOR);

        let mut catcher = 0;
        let mut allow_catch = true;
        for i in 0..self.items.len() {
            match self.items.get(i){
                Option::Some(item) => {
                    unsafe {
                        if item.on_mouse(self.mouse_point, mouse_event, allow_catch) {
                            allow_catch = false;
                            catcher = i;

                            self.redraw = max(self.redraw, REDRAW_ALL);
                        }
                    }
                },
                Option::None => ()
            }
        }

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
                    let i = (self.items.len() - 1 - reverse_i);
                    match self.items.get(i) {
                        Option::Some(item) => {
                            unsafe {
                                if ! item.draw(&self.display) {
                                    erase_i.push(i);
                                }
                            }
                        },
                        Option::None => ()
                    }
                }

                for i in erase_i.iter() {
                    drop(self.items.remove(*i));
                }
            }

            self.display.flip();

            self.display.cursor(self.mouse_point);

            self.redraw = REDRAW_NONE;
        }
    }

    pub fn handle_events(&mut self, events: &mut Vec<Event>){
        for event in events.iter() {
            match event.code {
                'm' => self.on_mouse(MouseEvent::from_event(event)),
                'k' => self.on_key(KeyEvent::from_event(event)),
                'r' => self.redraw = max(self.redraw, RedrawEvent::from_event(event).redraw),
                'o' => {
                    self.redraw = max(self.redraw, REDRAW_ALL);

                    let url_string = OpenEvent::from_event(event).url_string;
                    let url = URL::from_string(url_string.clone());

                    let mut found = false;
                    if url_string.ends_with(".md".to_string()) || url_string.ends_with(".rs".to_string()){
                        self.items.insert(0, box Editor::new());
                        found = true;
                    }else if url_string.ends_with(".bin".to_string()){
                        self.items.insert(0, box Executor::new());
                        found = true;
                    }else if url_string.ends_with(".bmp".to_string()){
                        self.items.insert(0, box Viewer::new());
                        found = true;
                    }else{
                        d("No program found: ");
                        url.d();
                        dl();
                    }

                    if found {
                        match self.items.get(0) {
                            Option::Some(item) => unsafe{
                                item.load(&url);
                            },
                            Option::None => ()
                        }
                    }
                }
                _ => ()
            }
        }
    }
}
