use programs::common::*;

pub struct WindowScheme;

pub struct WindowResourse {
    pub title: String,
    pub node: Node,
    pub vec: Vec<u8>,
    pub seek: usize,
    pub dirty: bool,
}



impl SessionItem for WindowScheme {
    fn scheme(&self) -> String {
        return "window".to_string();
    }

    fn open(&mut self, url: &URL) -> Box<Resource> {
        let scheme: String;
        let mut pointx :isize;
        let mut pointy :isize;
        let mut size_width :usize;
        let mut size_height :usize;
        let mut title :String;
        let mut split_url = url.string.split("/".to_string());
        scheme = match split_url.next() {
            Some(x) => x,
            None    => "".to_string(),
        }m
        pointx = match split_url.next() {
            Some(x) => x.to_num_signed(),
            None    => 0,
        };
        pointy = match split_url.next() {
            Some(y) => y.to_num_signed(),
            None    => 0,
        };
        size_width = match split_url.next() {
            Some(w) =>  w.to_num(),
            None    =>  10,
        };
        size_height = match split_url.next() {
            Some(h) =>  h.to_num(),
            None    =>  10,
        };
        title = match split_url.next() {
            Some(t) =>  t,
            None    =>  "Fail".to_string(),
        };
        let mut p: Point = Point::new(pointx, pointy);
        let mut s: Size = Size::new(size_width, size_height);
        
        let mut newWin = Window::new(p, s, title);

        unsafe {
            //newWin.ptr = newWin.deref_mut();
            let raw_win = Box::into_raw(newWin);
            
            //if raw_win.ptr as usize > 0 {
                (*::session_ptr).add_window(raw_win); 
            //}
        } 

        return box NoneResource; //TODO define a WindowResource
        //return box VecResource::new(URL::from_str("window://"),
        //                            ResourceType::File,
        //                            newWin);
    }
}

impl Drop for WindowScheme {
    
