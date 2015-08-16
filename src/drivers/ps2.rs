use common::pio::*;

use drivers::keyboard::keyboard_interrupt;
use drivers::mouse::mouse_interrupt;

use programs::common::*;

pub struct PS2;

impl SessionModule for PS2 {
    #[allow(unused_variables)]
    fn on_irq(&mut self, events: &mut Vec<URL>, irq: u8){
        if irq == 0x1 || irq == 0xC {
            self.on_poll(events);
        }
    }

    #[allow(unused_variables)]
    fn on_poll(&mut self, events: &mut Vec<URL>){
        unsafe{
            loop {
                let status = inb(0x64);
                if status & 0x21 == 1 {
                    let key_event = keyboard_interrupt();
                    if key_event.scancode > 0 {
                        let mut event = URL::new();
                        event.scheme = "k".to_string();
                        event.path.push(String::from_num(key_event.character as usize));
                        event.path.push(String::from_num(key_event.scancode as usize));
                        event.path.push(String::from_num(key_event.pressed as usize));
                        events.push(event);
                    }
                }else if status & 0x21 == 0x21 {
                    let mouse_event = mouse_interrupt();

                    if mouse_event.valid {
                        let mut event = URL::new();
                        event.scheme = "m".to_string();
                        event.path.push(String::from_num_signed(mouse_event.x));
                        event.path.push(String::from_num_signed(mouse_event.y));
                        event.path.push(String::from_num(mouse_event.left_button as usize));
                        event.path.push(String::from_num(mouse_event.middle_button as usize));
                        event.path.push(String::from_num(mouse_event.right_button as usize));
                        events.push(event);
                    }
                }else{
                    break;
                }
            }
        }
    }
}
