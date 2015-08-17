use common::pio::*;

use programs::common::*;

pub struct Serial {
    pub port: u16,
    pub irq: u8,
    pub escape: bool,
    pub cursor_control: bool
}

impl Serial {
    pub fn new(port: u16, irq: u8) -> Serial{
        return Serial {
            port: port,
            irq: irq,
            escape: false,
            cursor_control: false
        };
    }
}

impl SessionModule for Serial {
    fn on_irq(&mut self, irq: u8){
        if irq == self.irq {
            unsafe{
                while inb(self.port + 5) & 1 == 0 {}
                let mut c = inb(self.port) as char;
                let mut sc = 0;

                if self.escape {
                    self.escape = false;

                    if c == '['{
                        self.cursor_control = true;
                    }

                    c = '\0';
                }else if self.cursor_control {
                    self.cursor_control = false;

                    if c == 'A'{
                        sc = 0x48;
                    }else if c == 'B'{
                        sc = 0x50;
                    }else if c == 'C'{
                        sc = 0x4D;
                    }else if c == 'D'{
                        sc = 0x4B;
                    }

                    c = '\0';
                }else if c == '\x1B' {
                    self.escape = true;
                    c = '\0';
                }else if c == '\r' {
                    c = '\n';
                    dl();
                }else if c == '\x7F' {
                    c = '\x08';
                }else{
                    dc(c);
                }

                if c != '\0' || sc != 0 {
                    KeyEvent {
                        character: c,
                        scancode: sc,
                        pressed: true
                    }.trigger();
                }
            }
        }
    }
}
