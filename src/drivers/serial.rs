use common::debug::*;
use common::pio::*;

use drivers::keyboard::KeyEvent;

use programs::session::*;

pub struct Serial {
    pub port: u16,
    pub irq: u8
}

impl Serial {
    pub fn new(port: u16, irq: u8) -> Serial{
        return Serial {
            port: port,
            irq: irq
        };
    }
}

impl SessionDevice for Serial {
    #[allow(unused_variables)]
    fn on_irq(&mut self, session: &Session, updates: &mut SessionUpdates, irq: u8){
        if irq == self.irq {
            unsafe{
                while inb(self.port + 5) & 1 == 0 {}
                let mut c = inb(self.port) as char;

                dbh(c as u8);
                dl();

                if c == '\r' {
                    c = '\n';
                }else if c == '\x7F' {
                    c = '\x08';
                }
                updates.key_events.push(KeyEvent {
                    character: c,
                    scancode: 0,
                    pressed: true
                });
            }
        }
    }
}
