#![feature(alloc)]
#![feature(asm)]
#![feature(heap_api)]
#![feature(question_mark)]

extern crate alloc;
extern crate ransid;
extern crate syscall;

use std::cell::RefCell;
use std::collections::VecDeque;
use std::fs::File;
use std::io::{Read, Write};
use std::{slice, thread};
use ransid::{Console, Event};
use syscall::{physmap, physunmap, Packet, Result, Scheme, EVENT_READ, MAP_WRITE, MAP_WRITE_COMBINE};

use display::Display;
use mode_info::VBEModeInfo;
use primitive::fast_set64;

pub mod display;
pub mod mode_info;
pub mod primitive;

struct DisplayScheme {
    console: RefCell<Console>,
    display: RefCell<Display>,
    input: RefCell<VecDeque<u8>>,
    requested: RefCell<usize>
}

impl Scheme for DisplayScheme {
    fn open(&self, path: &[u8], _flags: usize) -> Result<usize> {
        if path == b"input" {
            Ok(1)
        } else {
            Ok(0)
        }
    }

    fn dup(&self, id: usize) -> Result<usize> {
        Ok(id)
    }

    fn fevent(&self, _id: usize, flags: usize) -> Result<usize> {
        *self.requested.borrow_mut() = flags;
        println!("fevent {:X}", flags);
        Ok(0)
    }

    fn fsync(&self, _id: usize) -> Result<usize> {
        Ok(0)
    }

    fn read(&self, _id: usize, buf: &mut [u8]) -> Result<usize> {
        let mut i = 0;
        let mut input = self.input.borrow_mut();
        while i < buf.len() && ! input.is_empty() {
            buf[i] = input.pop_front().unwrap();
            i += 1;
        }
        Ok(i)
    }

    fn write(&self, id: usize, buf: &[u8]) -> Result<usize> {
        if id == 1 {
            let mut input = self.input.borrow_mut();
            for &b in buf.iter() {
                input.push_back(b);
            }
            Ok(buf.len())
        } else {
            let mut display = self.display.borrow_mut();
            let mut console = self.console.borrow_mut();
            if console.cursor && console.x < console.w && console.y < console.h {
                display.rect(console.x * 8, console.y * 16, 8, 16, 0);
            }
            console.write(buf, |event| {
                match event {
                    Event::Char { x, y, c, color, bold, .. } => display.char(x * 8, y * 16, c, color.data, bold, false),
                    Event::Rect { x, y, w, h, color } => display.rect(x * 8, y * 16, w * 8, h * 16, color.data),
                    Event::Scroll { rows, color } => display.scroll(rows * 16, color.data)
                }
            });
            if console.cursor && console.x < console.w && console.y < console.h {
                display.rect(console.x * 8, console.y * 16, 8, 16, 0xFFFFFF);
            }
            Ok(buf.len())
        }
    }

    fn close(&self, _id: usize) -> Result<usize> {
        Ok(0)
    }
}

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
                input: RefCell::new(VecDeque::new()),
                requested: RefCell::new(0)
            };

            let mut blocked = VecDeque::new();
            loop {
                let mut packet = Packet::default();
                socket.read(&mut packet).expect("vesad: failed to read display scheme");
                //println!("vesad: {:?}", packet);

                // If it is a read packet, and there is no data, block it. Otherwise, handle packet
                if packet.a == syscall::number::SYS_READ && packet.d > 0 && scheme.input.borrow().is_empty() {
                    blocked.push_back(packet);
                } else {
                    scheme.handle(&mut packet);
                    socket.write(&packet).expect("vesad: failed to write display scheme");
                }

                // If there are blocked readers, and data is available, handle them
                while ! scheme.input.borrow().is_empty() {
                    if let Some(mut packet) = blocked.pop_front() {
                        scheme.handle(&mut packet);
                        socket.write(&packet).expect("vesad: failed to write display scheme");
                    } else {
                        break;
                    }
                }

                // If there are requested events, and data is available, send a notification
                if ! scheme.input.borrow().is_empty() && *scheme.requested.borrow() & EVENT_READ == EVENT_READ {
                    let event_packet = Packet {
                        id: 0,
                        a: syscall::number::SYS_FEVENT,
                        b: 0,
                        c: EVENT_READ,
                        d: scheme.input.borrow().len()
                    };
                    socket.write(&event_packet).expect("vesad: failed to write display scheme");
                }
            }
        });
    }
}
