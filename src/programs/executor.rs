use core::atomic::*;

use common::elf::*;

use programs::common::*;

pub struct Executor {
    executable: ELF,
    loading: bool,
    mapped: AtomicUsize,
    entry: usize,
    draw: usize,
    on_key: usize,
    on_mouse: usize
}

impl Executor {
    unsafe fn entry(&mut self){
        if self.executable.can_call(self.entry){
            //Rediculous call mechanism
            self.unsafe_map();
            let fn_ptr: *const usize = &self.entry;
            (*(fn_ptr as *const fn()))();
            self.unsafe_unmap();
        }
    }

    unsafe fn unsafe_map(&mut self){
        let mapped = self.mapped.fetch_add(1, Ordering::SeqCst);
        if self.executable.data > 0 && mapped == 0{
            self.executable.map();
        }
    }

    unsafe fn unsafe_unmap(&mut self){
        let mapped = self.mapped.fetch_sub(1, Ordering::SeqCst);
        if self.executable.data > 0 && mapped == 1{
            self.executable.unmap();
        }
    }
}

impl SessionItem for Executor {
    fn new() -> Executor {
        Executor {
            executable: ELF::new(),
            loading: false,
            mapped: AtomicUsize::new(0),
            entry: 0,
            draw: 0,
            on_mouse: 0,
            on_key: 0
        }
    }

    #[allow(unused_variables)]
    fn load(&mut self, url: &URL){
        self.loading = true;

        let self_ptr: *mut Executor = self;
        url.open_async(box move |mut resource: Box<Resource>|{
            let executor;
            unsafe{
                executor = &mut *self_ptr;
            }

            let mut vec: Vec<u8> = Vec::new();
            match resource.read_to_end(&mut vec){
                Option::Some(0) => (),
                Option::Some(len) => {
                    unsafe{
                        executor.executable = ELF::from_data(vec.as_ptr() as usize);
                        //self.executable.d();

                        executor.entry = executor.executable.entry();
                        executor.draw = executor.executable.symbol("draw".to_string());
                        executor.on_key = executor.executable.symbol("on_key".to_string());
                        executor.on_mouse = executor.executable.symbol("on_mouse".to_string());

                        executor.entry();

                        executor.loading = false;
                    }
                },
                Option::None => ()
            }
        });
    }

    fn draw(&mut self, display: &Display, events: &mut Vec<URL>) -> bool{
        unsafe {
            if self.executable.can_call(self.draw){
                //Rediculous call mechanism
                self.unsafe_map();
                let fn_ptr: *const usize = &self.draw;
                let ret = (*(fn_ptr as *const fn(&Display, &mut Vec<URL>) -> bool))(display, events);
                self.unsafe_unmap();

                return ret;
            }
        }
        return self.loading;
    }

    fn on_key(&mut self, events: &mut Vec<URL>, key_event: KeyEvent){
        unsafe {
            if self.executable.can_call(self.on_key){
                //Rediculous call mechanism
                self.unsafe_map();
                let fn_ptr: *const usize = &self.on_key;
                (*(fn_ptr as *const fn(&mut Vec<URL>, KeyEvent)))(events, key_event);
                self.unsafe_unmap();
            }
        }
    }

    fn on_mouse(&mut self, events: &mut Vec<URL>, mouse_point: Point, mouse_event: MouseEvent, allow_catch: bool) -> bool{
        unsafe {
            if self.executable.can_call(self.on_mouse){
                //Rediculous call mechanism
                self.unsafe_map();
                let fn_ptr: *const usize = &self.on_mouse;
                let ret = (*(fn_ptr as *const fn(&mut Vec<URL>, Point, MouseEvent, bool) -> bool))(events, mouse_point, mouse_event, allow_catch);
                self.unsafe_unmap();
                return ret;
            }
        }
        return false;
    }
}
