use common::context::*;
use common::memory::*;

use graphics::bmp::*;
use graphics::color::*;
use graphics::size::*;

use programs::common::*;
use programs::editor::*;
use programs::executor::*;
use programs::filemanager::*;
use programs::viewer::*;

pub struct Session {
    pub display: Display,
    pub background: BMP,
    pub cursor: BMP,
    pub mouse_point: Point,
    last_mouse_event: MouseEvent,
    pub items: Vec<Arc<SessionItem>>,
    pub redraw: usize
}

impl Session {
    pub fn new() -> Session {
        unsafe {
            Session {
                display: Display::root(),
                background: BMP::new(),
                cursor: BMP::new(),
                mouse_point: Point::new(0, 0),
                last_mouse_event: MouseEvent {
                    x: 0,
                    y: 0,
                    left_button: false,
                    middle_button: false,
                    right_button: false,
                    valid: false
                },
                items: Vec::new(),
                redraw: REDRAW_ALL
            }
        }
    }

    pub fn on_irq(&mut self, irq: u8){
        for item in self.items.iter() {
            unsafe{
                Arc::unsafe_get_mut(item).on_irq(irq);
            }
        }
    }

    pub fn on_poll(&mut self){
        for item in self.items.iter() {
            unsafe{
                Arc::unsafe_get_mut(item).on_poll();
            }
        }
    }

    pub fn open(&self, url: &URL) -> Box<Resource>{
        if url.scheme.len() == 0 {
            let mut list = String::new();

            for item in self.items.iter() {
                let scheme = item.scheme();
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
            for item in self.items.iter() {
                if item.scheme() == url.scheme {
                    unsafe{
                        return Arc::unsafe_get_mut(item).open(url);
                    }
                }
            }
            return box NoneResource;
        }
    }

    fn on_key(&mut self, key_event: KeyEvent){
        match self.items.get(0){
            Option::Some(item) => {
                unsafe{
                    Arc::unsafe_get_mut(item).on_key(key_event);
                }

                self.redraw = max(self.redraw, REDRAW_ALL);
            },
            Option::None => ()
        }
    }

    fn on_mouse(&mut self, mouse_event: MouseEvent){
        self.redraw = max(self.redraw, REDRAW_CURSOR);

        let mut catcher = 0;
        let mut allow_catch = true;
        for i in 0..self.items.len() {
            match self.items.get(i){
                Option::Some(item) => {
                    unsafe{
                        if Arc::unsafe_get_mut(item).on_mouse(mouse_event, allow_catch) {
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
                Option::Some(item) => self.items.insert(0, item),
                Option::None => ()
            }
        }

        //Not caught, can be caught by task bar
        if allow_catch {
            if mouse_event.left_button && !self.last_mouse_event.left_button && mouse_event.y <= 16 {
                self.items.insert(0, Arc::new(FileManager::new()));
                self.redraw = max(self.redraw, REDRAW_ALL);
            }
        }

        self.last_mouse_event = mouse_event;
    }

    pub fn redraw(&mut self){
        if self.redraw > REDRAW_NONE {
            if self.redraw >= REDRAW_ALL {
                self.display.set(Color::new(64, 64, 64));
                if self.background.data > 0 {
                    self.display.image(Point::new((self.display.width as isize - self.background.size.width as isize)/2, (self.display.height as isize - self.background.size.height as isize)/2), self.background.data, self.background.size);
                }

                self.display.rect(Point::new(0, 0), Size::new(self.display.width, 18), Color::new(0, 0, 0));
                self.display.text(Point::new(self.display.width as isize/ 2 - 3*8, 1), &String::from_str("Redox"), Color::new(255, 255, 255));

                let mut erase_i: Vec<usize> = Vec::new();
                for reverse_i in 0..self.items.len() {
                    let i = self.items.len() - 1 - reverse_i;
                    match self.items.get(i) {
                        Option::Some(item) => if ! item.draw(&self.display) {
                            erase_i.push(i);
                        },
                        Option::None => ()
                    }
                }

                for i in erase_i.iter() {
                    drop(self.items.remove(*i));
                }
            }

            self.display.flip();

            if self.cursor.data > 0 {
                self.display.image_alpha_onscreen(self.mouse_point, self.cursor.data, self.cursor.size);
            }else{
                self.display.char_onscreen(Point::new(self.mouse_point.x - 3, self.mouse_point.y - 9), 'X', Color::new(255, 255, 255));
            }

            self.redraw = REDRAW_NONE;
        }
    }

    pub fn event(&mut self, mut event: Event){
        match event.code {
            'm' => self.on_mouse(MouseEvent::from_event(&mut event)),
            'k' => self.on_key(KeyEvent::from_event(&mut event)),
            'r' => self.redraw = max(self.redraw, RedrawEvent::from_event(&mut event).redraw),
            'o' => {
                self.redraw = max(self.redraw, REDRAW_ALL);

                let url_string = OpenEvent::from_event(&mut event).url_string;
                let url = URL::from_string(url_string.clone());

                let mut found = false;
                if url_string.ends_with(".md".to_string()) || url_string.ends_with(".rs".to_string()) || url_string.ends_with(".sh".to_string()){
                    self.items.insert(0, Arc::new(Editor::new()));
                    found = true;
                }else if url_string.ends_with(".bin".to_string()){
                    self.items.insert(0, Arc::new(Executor::new()));
                    found = true;
                }else if url_string.ends_with(".bmp".to_string()){
                    self.items.insert(0, Arc::new(Viewer::new()));
                    found = true;
                }else{
                    d("No program found: ");
                    url.d();
                    dl();
                }

                if found {
                    match self.items.get(0) {
                        Option::Some(item) => {
                            unsafe{
                                let item_ptr: *mut Arc<SessionItem> = alloc_type();
                                ptr::write(item_ptr, item.clone());

                                let url_ptr: *mut URL = alloc_type();
                                ptr::write(url_ptr, url.clone());

                                let mut item_main_args: Vec<usize> = Vec::new();
                                item_main_args.push(url_ptr as usize);
                                item_main_args.push(item_ptr as usize);
                                (*::contexts_ptr).push(Context::new(item_main as usize, &item_main_args));
                            }
                        }
                        Option::None => ()
                    }
                }
            }
            _ => ()
        }
    }
}
