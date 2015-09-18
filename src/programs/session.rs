use common::context::*;
use common::scheduler::*;

use graphics::bmp::*;

use programs::common::*;
use programs::editor::*;
use programs::executor::*;
use programs::filemanager::*;
use programs::player::*;
use programs::viewer::*;

pub struct Session {
    pub display: Display,
    pub background: BMP,
    pub cursor: BMP,
    pub icon: BMP,
    pub mouse_point: Point,
    last_mouse_event: MouseEvent,
    pub items: Vec<Box<SessionItem>>,
    pub windows: Vec<*mut Window>,
    pub windows_ordered: Vec<*mut Window>,
    pub redraw: usize
}

impl Session {
    pub fn new() -> Session {
        unsafe {
            Session {
                display: Display::root(),
                background: BMP::new(),
                cursor: BMP::new(),
                icon: BMP::new(),
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
                windows: Vec::new(),
                windows_ordered: Vec::new(),
                redraw: REDRAW_ALL
            }
        }
    }

    pub unsafe fn add_window(&mut self, add_window_ptr: *mut Window){
        self.windows.push(add_window_ptr);
        self.windows_ordered.push(add_window_ptr);
        self.redraw = max(self.redraw, REDRAW_ALL);
    }

    pub unsafe fn remove_window(&mut self, remove_window_ptr: *mut Window){
        let mut i = 0;
        while i < self.windows.len() {
            let mut remove = false;

            match self.windows.get(i) {
                Option::Some(window_ptr) => if *window_ptr == remove_window_ptr {
                    remove = true;
                }else{
                    i += 1;
                },
                Option::None => break
            }

            if remove {
                self.windows.remove(i);
                self.redraw = max(self.redraw, REDRAW_ALL);
            }
        }

        i = 0;
        while i < self.windows_ordered.len() {
            let mut remove = false;

            match self.windows_ordered.get(i) {
                Option::Some(window_ptr) => if *window_ptr == remove_window_ptr {
                    remove = true;
                }else{
                    i += 1;
                },
                Option::None => break
            }

            if remove {
                self.windows_ordered.remove(i);
                self.redraw = max(self.redraw, REDRAW_ALL);
            }
        }
    }

    pub unsafe fn on_irq(&mut self, irq: u8){
        for item in self.items.iter() {
            let reenable = start_no_ints();
            item.on_irq(irq);
            end_no_ints(reenable);
        }
    }

    pub unsafe fn on_poll(&mut self){
        for item in self.items.iter() {
            let reenable = start_no_ints();
            item.on_poll();
            end_no_ints(reenable);
        }
    }

    pub fn open(&self, url: &URL) -> Box<Resource>{
        if url.scheme().len() == 0 {
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

            return box VecResource::new(URL::new(), ResourceType::Dir, list.to_utf8());
        }else{
            for item in self.items.iter() {
                if item.scheme() == url.scheme() {
                    return item.open(url);
                }
            }
            return box NoneResource;
        }
    }

    fn item_main(&mut self, mut item: Box<SessionItem>, url: URL){
        Context::spawn(box move ||{
            item.main(url);
        });
    }

    fn on_key(&mut self, key_event: KeyEvent){
        if self.windows.len() > 0 {
            match self.windows.get(self.windows.len() - 1){
                Option::Some(window_ptr) => {
                    unsafe{
                        (**window_ptr).on_key(key_event);
                        self.redraw = max(self.redraw, REDRAW_ALL);
                    }
                },
                Option::None => ()
            }
        }
    }

    fn on_mouse(&mut self, mouse_event: MouseEvent){
        let mut catcher = -1;

        if mouse_event.y >= self.display.height as isize - 32 {
            if mouse_event.left_button &&  !self.last_mouse_event.left_button {
                if mouse_event.x <= 56 {
                    self.item_main(box FileManager::new(), URL::from_string(&"file:///".to_string()));
                }else{
                    let mut chars = 32;
                    while chars > 4 && (chars*8 + 3*4) * self.windows.len() > self.display.width {
                        chars -= 1;
                    }

                    for i in 0..self.windows_ordered.len() {
                        let x = (5*8 + 2*8 + (chars*8 + 3*4) * i) as isize;
                        let w = (chars*8 + 2*4) as usize;
                        if mouse_event.x >= x && mouse_event.x < x + w as isize {
                            match self.windows_ordered.get(i) {
                                Option::Some(window_ptr) => unsafe {
                                    for j in 0..self.windows.len() {
                                        match self.windows.get(j){
                                            Option::Some(catcher_window_ptr) => if catcher_window_ptr == window_ptr {
                                                if j == self.windows.len() - 1 {
                                                    (**window_ptr).minimized = !(**window_ptr).minimized;
                                                }else{
                                                    catcher = j as isize;
                                                    (**window_ptr).minimized = false;
                                                }
                                                break;
                                            },
                                            Option::None => break
                                        }
                                    }
                                    self.redraw = max(self.redraw, REDRAW_ALL);
                                },
                                Option::None => ()
                            }
                            break;
                        }
                    }
                }
            }
        }else{
            for reverse_i in 0..self.windows.len() {
                let i = self.windows.len() - 1 - reverse_i;
                match self.windows.get(i){
                    Option::Some(window_ptr) => unsafe{
                        if (**window_ptr).on_mouse(mouse_event, catcher < 0) {
                            catcher = i as isize;

                            self.redraw = max(self.redraw, REDRAW_ALL);
                        }
                    },
                    Option::None => ()
                }
            }
        }

        if catcher >= 0 && catcher < self.windows.len() as isize - 1 {
            match self.windows.remove(catcher as usize){
                Option::Some(window_ptr) => self.windows.push(window_ptr),
                Option::None => ()
            }
        }

        self.last_mouse_event = mouse_event;
    }

    pub unsafe fn redraw(&mut self){
        if self.redraw > REDRAW_NONE {
            if self.redraw >= REDRAW_ALL {
                self.display.set(Color::new(64, 64, 64));
                if self.background.data > 0 {
                    self.display.image(Point::new((self.display.width as isize - self.background.size.width as isize)/2, (self.display.height as isize - self.background.size.height as isize)/2), self.background.data, self.background.size);
                }

                for i in 0..self.windows.len(){
                    match self.windows.get(i) {
                        Option::Some(window_ptr) => {
                            (**window_ptr).focused = i == self.windows.len() - 1;
                            (**window_ptr).draw(&self.display);
                        },
                        Option::None => ()
                    }
                }

                self.display.rect(Point::new(0, self.display.height as isize - 32), Size::new(self.display.width, 32), Color::new(0, 0, 0));
                if self.icon.data > 0 {
                    self.display.image_alpha(Point::new(12, self.display.height as isize - 32), self.icon.data, self.icon.size);
                }else{
                    self.display.text(Point::new(8, self.display.height as isize - 24), &String::from_str("Redox"), Color::new(255, 255, 255));
                }

                let mut chars = 32;
                while chars > 4 && (chars*8 + 3*4) * self.windows.len() > self.display.width {
                    chars -= 1;
                }

                for i in 0..self.windows_ordered.len() {
                    match self.windows_ordered.get(i) {
                        Option::Some(window_ptr) => {
                            let x = (5*8 + 2*8 + (chars*8 + 3*4) * i) as isize;
                            let w = (chars*8 + 2*4) as usize;
                            self.display.rect(Point::new(x, self.display.height as isize - 32), Size::new(w, 32), (**window_ptr).border_color);
                            self.display.text(Point::new(x + 4, self.display.height as isize - 24), &(**window_ptr).title.substr(0, chars as usize), (**window_ptr).title_color);
                        },
                        Option::None => ()
                    }
                }
            }

            let reenable = start_no_ints();

            self.display.flip();

            if self.cursor.data > 0 {
                self.display.image_alpha_onscreen(self.mouse_point, self.cursor.data, self.cursor.size);
            }else{
                self.display.char_onscreen(Point::new(self.mouse_point.x - 3, self.mouse_point.y - 9), 'X', Color::new(255, 255, 255));
            }

            self.redraw = REDRAW_NONE;

            end_no_ints(reenable);
        }
    }

    pub fn event(&mut self, event: Event){
        match event.to_option() {
            EventOption::Mouse(mouse_event) => self.on_mouse(mouse_event),
            EventOption::Key(key_event) => self.on_key(key_event),
            EventOption::Redraw(redraw_event) => self.redraw = max(self.redraw, redraw_event.redraw),
            EventOption::Open(open_event) => {
                let url_string = open_event.url_string;
                let url = URL::from_string(&url_string);

                if url_string.ends_with(".md".to_string()) || url_string.ends_with(".rs".to_string()) || url_string.ends_with(".sh".to_string()){
                    self.item_main(box Editor::new(), url);
                }else if url_string.ends_with(".bin".to_string()){
                    self.item_main(box Executor::new(), url);
                }else if url_string.ends_with(".wav".to_string()){
                    self.item_main(box Player::new(), url);
                }else if url_string.ends_with(".bmp".to_string()){
                    self.item_main(box Viewer::new(), url);
                }else{
                    d("No program found: ");
                    url.d();
                    dl();
                }
            }
            _ => ()
        }
    }
}
