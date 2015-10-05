use alloc::boxed::Box;

use programs::common::*;
use graphics::point::*;
use graphics::size::*;
use graphics::window::*;
use common::string::*;
use common::resource::*;

use core::ops::DerefMut;

pub struct WindowScheme {
    pub current_window: *mut Window,
}

pub struct WindowResource {
    pub active_window: Box<Window>,
}

impl Resource for WindowResource {
     //Required functions
    /// Return the url of this resource
    fn url(&self) -> URL {
        return URL::from_string(&("window://test".to_string()));
    } 
    /// Return the type of this resource
    fn stat(&self) -> ResourceType {
        return ResourceType::Window;
    }
    /// Read data to buffer
    fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        //TODO implement
        return Option::None;
    }
    /// Write to resource
    fn write(&mut self, buf: &[u8]) -> Option<usize> {
        //TODO implement
        return Option::None;
    }
    /// Seek
    fn seek(&mut self, pos: ResourceSeek) -> Option<usize> {
        return Option::None; //TODO implement
    }
    /// Sync the resource
    fn sync(&mut self) -> bool {
        return true;
    }
}
 




impl SessionItem for WindowScheme {
    fn scheme(&self) -> String {
        return "window".to_string();
    }

    fn open(&mut self, url: &URL) -> Box<Resource> {
        let scheme :String;
        let mut pointx :isize;
        let mut pointy :isize;
        let mut size_width :usize;
        let mut size_height :usize;
        let mut title :String;

        //window://host/path/path/path is the path type we're working with.
        let mut url_path = url.path_parts();
        pointx = match url_path.get(0) {
            Some(x) => x.to_num_signed(),
            None    => 0,
        };
        pointy = match url_path.get(1) {
            Some(y) => y.to_num_signed(),
            None    => 0,
        };
        size_width = match url_path.get(2) {
            Some(w) =>  w.to_num(),
            None    =>  10,
        };
        size_height = match url_path.get(3) {
            Some(h) =>  h.to_num(),
            None    =>  10,
        };
        title = match url_path.get(4) {
            Some(t) =>  t.clone(),
            None    =>  "Fail".to_string(),
        };
        let mut p: Point = Point::new(pointx, pointy);
        let mut s: Size = Size::new(size_width, size_height);
        let mut newWin  = Window::new(p, s, title);
        unsafe {
            newWin.ptr = newWin.deref_mut();
            self.current_window = newWin.ptr;
            //self.raw_current = Box::into_raw(newWin);
            if newWin.ptr as usize > 0 {
                (*::session_ptr).add_window(self.current_window); 
            }
        }
        
        return box WindowResource {
            active_window : newWin,
        };
        //return box VecResource::new(URL::from_str("window://"),
        //                            ResourceType::File,
        //                            newWin);
    }
}

impl Drop for WindowScheme {
   fn drop(&mut self) {
       unsafe {
           (*::session_ptr).remove_window(self.current_window);
       }
   }
}
