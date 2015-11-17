use alloc::boxed::Box;

use common::event;

use drivers::pio::*;

use schemes::KScheme;

#[repr(packed)]
struct SerialInfo {
    pub ports: [u16; 4]
}

const SERIALINFO: *const SerialInfo = 0x400 as *const SerialInfo;

/// Serial
pub struct Serial {
    pub data: Pio8,
    pub status: Pio8,
    pub irq: u8,
    pub escape: bool,
    pub cursor_control: bool,
}

impl Serial {
    /// Initialize serial ports
    pub fn init() {
        //     Pio8::new(0x3F8 + 1).write(0x00);
        //     Pio8::new(0x3F8 + 3).write(0x80);
        //     Pio8::new(0x3F8 + 0).write(0x03);
        //     Pio8::new(0x3F8 + 1).write(0x00);
        //     Pio8::new(0x3F8 + 3).write(0x03);
        //     Pio8::new(0x3F8 + 2).write(0xC7);
        //     Pio8::new(0x3F8 + 4).write(0x0B);
        //     Pio8::new(0x3F8 + 1).write(0x01);
    }

    /// Create new
    pub fn new(port: u16, irq: u8) -> Box<Self> {
        box Serial {
            data: Pio8::new(port),
            status: Pio8::new(port + 5),
            irq: irq,
            escape: false,
            cursor_control: false,
        }
    }
}

impl KScheme for Serial {
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
