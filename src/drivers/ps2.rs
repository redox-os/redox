use common::pio::*;

use drivers::keyboard::*;
use drivers::mouse::*;

use programs::session::*;

pub struct PS2;

impl SessionModule for PS2 {
    #[allow(unused_variables)]
    fn on_irq(&mut self, session: &Session, updates: &mut SessionUpdates, irq: u8){
        if irq == 0x1 || irq == 0xC {
            self.on_poll(session, updates);
        }
    }

    #[allow(unused_variables)]
    fn on_poll(&mut self, session: &Session, updates: &mut SessionUpdates){
        unsafe{
            loop {
                let status = inb(0x64);
                if status & 0x21 == 1 {
                    let key_event = keyboard_interrupt();
                    if key_event.scancode > 0 {
                        updates.events.push(box key_event);
                    }
                }else if status & 0x21 == 0x21 {
                    let mouse_event = mouse_interrupt();

                    if mouse_event.valid {
                        updates.events.push(box mouse_event);
                    }
                }else{
                    break;
                }
            }
        }
    }
}
