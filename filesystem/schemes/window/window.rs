use redox::*;

/// A window scheme
pub struct Scheme;

/// A window resource
pub struct Resource {
    /// The window
    pub window: Box<Window>,
    /// Seek point
    pub seek: usize,
}

impl Resource {
    pub fn dup(&self) -> Option<Box<Resource>> {
        Some(box Resource {
            window: orbital::window::Window::new(self.window.point.x, self.window.point.y, self.window.size.width, self.window.size.height, &self.window.t).unwrap(),
            seek: self.seek,
        })
    }

    /// Return the path of this resource
    // TODO: this should be unique
    pub fn path(&self) -> Option<String> {
        return Some(format!("window://{}/{}/{}/{}/{}",
                       self.window.point.x,
                       self.window.point.y,
                       self.window.size.width,
                       self.window.size.height,
                       self.window.t));
    }

    /// Read data to buffer
    pub fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        //Read events from window
        let mut i = 0;
        while buf.len() - i >= mem::size_of::<Event>() {
            match self.window.poll() {
                Some(event) => {
                    unsafe { ptr::write(buf.as_ptr().offset(i as isize) as *mut Event, event) };
                    i += mem::size_of::<Event>();
                },
                None => break,
            }
        }

        Some(i)
    }

    /// Write to resource
    pub fn write(&mut self, buf: &[u8]) -> Option<usize> {
        let content = &mut self.window.content;

        let size = cmp::min(content.size - self.seek, buf.len());
        unsafe {
            Display::copy_run(buf.as_ptr() as usize,
                              content.offscreen + self.seek,
                              size);
        }
        self.seek += size;

        return Some(size);
    }

    /// Seek
    pub fn seek(&mut self, pos: SeekFrom) -> Option<usize> {
        let end = self.window.content.size;

        self.seek = match pos {
            SeekFrom::Start(offset) => cmp::min(end, cmp::max(0, offset)),
            SeekFrom::Current(offset) => cmp::min(end, cmp::max(0, self.seek as isize + offset) as usize),
            SeekFrom::End(offset) => cmp::min(end, cmp::max(0, end as isize + offset) as usize),
        };

        return Some(self.seek);
    }

    /// Sync the resource, should flip
    pub fn sync(&mut self) -> bool {
        self.window.redraw();
        true
    }
}

impl Scheme {
    pub fn new() -> Box<Self> {
        box Scheme
    }

    pub fn open(&mut self, path: &str) -> Option<Box<Resource>> {
        //window://host/path/path/path is the path type we're working with.
        // TODO: use same parsing as in kernel space?
        let url_path = URL::from_string(&path.to_string()).path_parts();
        let pointx = match url_path.get(0) {
            Some(x) => x.to_num_signed(),
            None => 0,
        };
        let pointy = match url_path.get(1) {
            Some(y) => y.to_num_signed(),
            None => 0,
        };
        let size_width = match url_path.get(2) {
            Some(w) => w.to_num(),
            None => 100,
        };
        let size_height = match url_path.get(3) {
            Some(h) => h.to_num(),
            None => 100,
        };

        let mut title = match url_path.get(4) {
            Some(t) => t.clone(),
            None => String::new(),
        };
        for i in 5..url_path.len() {
            if let Some(t) = url_path.get(i) {
                title = title + "/" + t;
            }
        }

        Some(box Resource {
            window: orbital::window::Window::new(pointx, pointy, size_width, size_height, &title[..]).unwrap(),
            seek: 0,
        })
    }
}
