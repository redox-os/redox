use common::context::*;
use common::scheduler::*;

use graphics::bmp::*;

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
    pub items: Vec<Box<SessionItem>>,
    pub windows: Vec<*mut Window>,
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
                windows: Vec::new(),
                redraw: REDRAW_ALL
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
        match self.windows.get(0){
            Option::Some(window_ptr) => {
                unsafe{
                    (**window_ptr).on_key(key_event);
                    self.redraw = max(self.redraw, REDRAW_ALL);
                }
            },
            Option::None => ()
        }
    }

    fn on_mouse(&mut self, mouse_event: MouseEvent){
        let mut catcher = 0;
        let mut allow_catch = true;
        for i in 0..self.windows.len() {
            match self.windows.get(i){
                Option::Some(window_ptr) => unsafe{
                    if (**window_ptr).on_mouse(mouse_event, allow_catch) {
                        allow_catch = false;
                        catcher = i;

                        self.redraw = max(self.redraw, REDRAW_ALL);
                    }
                },
                Option::None => ()
            }
        }

        if catcher > 0 && catcher < self.windows.len() {
            match self.windows.remove(catcher){
                Option::Some(window_ptr) => self.windows.insert(0, window_ptr),
                Option::None => ()
            }
        }

        //Not caught, can be caught by task bar
        if allow_catch {
            if mouse_event.left_button && !self.last_mouse_event.left_button && mouse_event.y <= 16 {
                self.item_main(box FileManager::new(), URL::from_string(&"file:///".to_string()));
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

                self.display.rect(Point::new(0, 0), Size::new(self.display.width, 18), Color::new(0, 0, 0));
                self.display.text(Point::new(self.display.width as isize/ 2 - 3*8, 1), &String::from_str("Redox"), Color::new(255, 255, 255));

                for reverse_i in 0..self.windows.len(){
                    let i = self.windows.len() - 1 - reverse_i;
                    match self.windows.get(i) {
                        Option::Some(window_ptr) => (**window_ptr).draw(&self.display),
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
