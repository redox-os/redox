#![feature(alloc)]
#![feature(asm)]
#![feature(heap_api)]
#![feature(question_mark)]

extern crate alloc;
extern crate ransid;
extern crate syscall;

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Read, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{cmp, slice, thread};
use ransid::{Console, Event};
use syscall::{physmap, physunmap, Error, EINVAL, EBADF, Packet, Result, Scheme, SEEK_SET, SEEK_CUR, SEEK_END, MAP_WRITE, MAP_WRITE_COMBINE};

use display::Display;
use mode_info::VBEModeInfo;
use primitive::{fast_copy, fast_set64};

pub mod display;
pub mod mode_info;
pub mod primitive;

struct DisplayScheme {
    console: RefCell<Console>,
    display: RefCell<Display>,
    next_id: AtomicUsize,
    handles: RefCell<BTreeMap<usize, usize>>
}

impl Scheme for DisplayScheme {
    fn open(&self, _path: &[u8], _flags: usize) -> Result<usize> {
        Ok(0)
    }

    fn dup(&self, _id: usize) -> Result<usize> {
        Ok(0)
    }

    fn write(&self, _id: usize, buf: &[u8]) -> Result<usize> {
        let mut display = self.display.borrow_mut();
        self.console.borrow_mut().write(buf, |event| {
            match event {
                Event::Char { x, y, c, color, .. } => display.char(x * 8, y * 16, c, color.data),
                Event::Rect { x, y, w, h, color } => display.rect(x * 8, y * 16, w * 8, h * 16, color.data),
                Event::Scroll { rows, color } => display.scroll(rows * 16, color.data)
            }
        });
        Ok(buf.len())
    }

    fn close(&self, id: usize) -> Result<usize> {
        Ok(0)
    }
}

/*
struct DisplayScheme {
    display: RefCell<Display>,
    next_id: AtomicUsize,
    handles: RefCell<BTreeMap<usize, usize>>
}

impl Scheme for DisplayScheme {
    fn open(&self, _path: &[u8], _flags: usize) -> Result<usize> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        self.handles.borrow_mut().insert(id, 0);
        Ok(id)
    }

    fn write(&self, id: usize, buf: &[u8]) -> Result<usize> {
        let mut handles = self.handles.borrow_mut();
        let mut seek = handles.get_mut(&id).ok_or(Error::new(EBADF))?;
        let mut display = self.display.borrow_mut();
        let len = cmp::min(buf.len(), display.offscreen.len() * 4 - *seek);
        unsafe {
            fast_copy((display.offscreen.as_mut_ptr() as usize + *seek) as *mut u8, buf.as_ptr(), len);
            fast_copy((display.onscreen.as_mut_ptr() as usize + *seek) as *mut u8, buf.as_ptr(), len);
        }
        *seek += len;
        Ok(len)
    }

    fn seek(&self, id: usize, pos: usize, whence: usize) -> Result<usize> {
        let mut handles = self.handles.borrow_mut();
        let mut seek = handles.get_mut(&id).ok_or(Error::new(EBADF))?;
        let len = self.display.borrow().offscreen.len() * 4;
        *seek = match whence {
            SEEK_SET => cmp::min(len, pos),
            SEEK_CUR => cmp::max(0, cmp::min(len as isize, *seek as isize + pos as isize)) as usize,
            SEEK_END => cmp::max(0, cmp::min(len as isize, len as isize + pos as isize)) as usize,
            _ => return Err(Error::new(EINVAL))
        };
        Ok(*seek)
    }

    fn close(&self, id: usize) -> Result<usize> {
        self.handles.borrow_mut().remove(&id).ok_or(Error::new(EBADF)).and(Ok(0))
    }
}
*/

fn main() {
    let width;
    let height;
    let physbaseptr;

    {
        let mode_info = unsafe { &*(physmap(0x5200, 4096, 0).expect("vesad: failed to map VBE info") as *const VBEModeInfo) };

        width = mode_info.xresolution as usize;
        height = mode_info.yresolution as usize;
        physbaseptr = mode_info.physbaseptr as usize;

        unsafe { let _ = physunmap(mode_info as *const _ as usize); }
    }

    if physbaseptr > 0 {
        thread::spawn(move || {
            let mut socket = File::create(":display").expect("vesad: failed to create display scheme");

            let size = width * height;

            let onscreen = unsafe { physmap(physbaseptr as usize, size * 4, MAP_WRITE | MAP_WRITE_COMBINE).expect("vesad: failed to map VBE LFB") };
            unsafe { fast_set64(onscreen as *mut u64, 0, size/2) };

            let offscreen = unsafe { alloc::heap::allocate(size * 4, 4096) };
            unsafe { fast_set64(offscreen as *mut u64, 0, size/2) };

            let scheme = DisplayScheme {
                console: RefCell::new(Console::new(width/8, height/16)),
                display: RefCell::new(Display::new(width, height,
                    unsafe { slice::from_raw_parts_mut(onscreen as *mut u32, size) },
                    unsafe { slice::from_raw_parts_mut(offscreen as *mut u32, size) }
                )),
                next_id: AtomicUsize::new(0),
                handles: RefCell::new(BTreeMap::new())
            };

            loop {
                let mut packet = Packet::default();
                socket.read(&mut packet);
                //println!("vesad: {:?}", packet);
                scheme.handle(&mut packet);
                socket.write(&packet);
            }
        });
    }
}
