use common::pio::*;

use drivers::keyboard::keyboard_interrupt;
use drivers::mouse::mouse_interrupt;

use programs::common::*;

pub struct PS2;

impl SessionItem for PS2 {
    fn on_irq(&mut self, irq: u8){
        if irq == 0x1 || irq == 0xC {
            self.on_poll();
        }
    }

    fn on_poll(&mut self){
        unsafe{
            loop {
                let status = inb(0x64);
                if status & 0x21 == 1 {
                    let key_event = keyboard_interrupt();
                    if key_event.scancode > 0 {
                        key_event.trigger();
                    }
                }else if status & 0x21 == 0x21 {
                    let mouse_event = mouse_interrupt();

                    if mouse_event.valid {
                        mouse_event.trigger();
                    }
                }else{
                    break;
                }
            }
        }
    }
}
