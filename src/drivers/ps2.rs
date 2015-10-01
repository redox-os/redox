use drivers::keyboard::keyboard_interrupt;
use drivers::mouse::mouse_interrupt;
use drivers::pio::*;

use programs::common::*;

pub struct PS2 {
    status: PIO8
}

impl PS2 {
    pub fn new() -> PS2 {
        return PS2 {
            status: PIO8::new(0x64)
        };
    }
}

impl SessionItem for PS2 {
    fn on_irq(&mut self, irq: u8){
        if irq == 0x1 || irq == 0xC {
            self.on_poll();
        }
    }

    fn on_poll(&mut self){
        loop {
            let status = unsafe { self.status.read() };
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
