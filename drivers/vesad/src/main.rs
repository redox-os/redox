#![feature(alloc)]
#![feature(asm)]
#![feature(heap_api)]
#![feature(question_mark)]

extern crate alloc;
extern crate syscall;

use std::fs::File;
use std::io::{Read, Write};
use std::thread;
use syscall::{physmap, physunmap, Packet, Scheme, MAP_WRITE, MAP_WRITE_COMBINE};

use mode_info::VBEModeInfo;
use primitive::fast_set64;
use scheme::DisplayScheme;

pub mod display;
pub mod mode_info;
pub mod primitive;
pub mod scheme;
pub mod screen;

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

            let onscreen = unsafe { physmap(physbaseptr, size * 4, MAP_WRITE | MAP_WRITE_COMBINE).expect("vesad: failed to map VBE LFB") };
            unsafe { fast_set64(onscreen as *mut u64, 0, size/2) };

            let scheme = DisplayScheme::new(width, height, onscreen);

            let mut blocked = Vec::new();
            loop {
                let mut packet = Packet::default();
                socket.read(&mut packet).expect("vesad: failed to read display scheme");
                //println!("vesad: {:?}", packet);

                // If it is a read packet, and there is no data, block it. Otherwise, handle packet
                if packet.a == syscall::number::SYS_READ && packet.d > 0 && scheme.will_block(packet.b) {
                    blocked.push(packet);
                } else {
                    scheme.handle(&mut packet);
                    socket.write(&packet).expect("vesad: failed to write display scheme");
                }

                // If there are blocked readers, and data is available, handle them
                {
                    let mut i = 0;
                    while i < blocked.len() {
                        if ! scheme.will_block(blocked[i].b) {
                            let mut packet = blocked.remove(i);
                            scheme.handle(&mut packet);
                            socket.write(&packet).expect("vesad: failed to write display scheme");
                        } else {
                            i += 1;
                        }
                    }
                }

                // If there are requested events, and data is available, send a notification
                /* TODO
                if (! scheme.screen.borrow().input.is_empty() || scheme.screen.borrow().end_of_input) && scheme.screen.borrow().requested & EVENT_READ == EVENT_READ {
                    let event_packet = Packet {
                        id: 0,
                        pid: 0,
                        uid: 0,
                        gid: 0,
                        a: syscall::number::SYS_FEVENT,
                        b: 0,
                        c: EVENT_READ,
                        d: scheme.screen.borrow().input.len()
                    };
                    socket.write(&event_packet).expect("vesad: failed to write display scheme");
                }
                */
            }
        });
    }
}
