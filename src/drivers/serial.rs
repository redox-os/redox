use common::event;

use drivers::pio::*;

use programs::common::SessionItem;

pub struct Serial {
    pub data: PIO8,
    pub status: PIO8,
    pub irq: u8,
    pub escape: bool,
    pub cursor_control: bool,
}

impl Serial {
    pub fn new(port: u16, irq: u8) -> Serial {
        Serial {
            data: PIO8::new(port),
            status: PIO8::new(port + 5),
            irq: irq,
            escape: false,
            cursor_control: false,
        }
    }
}

impl SessionItem for Serial {
    fn on_irq(&mut self, irq: u8) {
        if irq == self.irq {
            while unsafe { self.status.read() } & 1 == 0 {
                break;
            }

            let mut c = unsafe { self.data.read() } as char;
            let mut sc = 0;

            if self.escape {
                self.escape = false;

                if c == '[' {
                    self.cursor_control = true;
                }

                c = '\0';
            } else if self.cursor_control {
                self.cursor_control = false;

                if c == 'A' {
                    sc = event::K_UP;
                } else if c == 'B' {
                    sc = event::K_DOWN;
                } else if c == 'C' {
                    sc = event::K_RIGHT;
                } else if c == 'D' {
                    sc = event::K_LEFT;
                }

                c = '\0';
            } else if c == '\x1B' {
                self.escape = true;
                c = '\0';
            } else if c == '\r' {
                c = '\n';
            } else if c == '\x7F' {
                sc = event::K_BKSP;
                c = '\0';
            }

            if c != '\0' || sc != 0 {
                event::KeyEvent {
                    character: c,
                    scancode: sc,
                    pressed: true,
                }
                    .trigger();
            }
        }
    }
}
