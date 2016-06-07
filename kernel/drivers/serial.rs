use alloc::boxed::Box;

use collections::string::String;

use common::event;

use drivers::io::{Io, Pio};

use fs::KScheme;

#[repr(packed)]
struct SerialInfo {
    pub ports: [u16; 4],
}

const SERIALINFO: *const SerialInfo = 0x400 as *const SerialInfo;

/// Serial
pub struct Serial {
    pub data: Pio<u8>,
    pub status: Pio<u8>,
    pub irq: u8,
    pub escape: bool,
    pub cursor_control: bool,
}

impl Serial {
    /// Create new
    pub fn new(port: u16, irq: u8) -> Box<Self> {
        Pio::<u8>::new(port + 1).write(0x00);
        Pio::<u8>::new(port + 3).write(0x80);
        Pio::<u8>::new(port + 0).write(0x03);
        Pio::<u8>::new(port + 1).write(0x00);
        Pio::<u8>::new(port + 3).write(0x03);
        Pio::<u8>::new(port + 2).write(0xC7);
        Pio::<u8>::new(port + 4).write(0x0B);
        Pio::<u8>::new(port + 1).write(0x01);

        box Serial {
            data: Pio::<u8>::new(port),
            status: Pio::<u8>::new(port + 5),
            irq: irq,
            escape: false,
            cursor_control: false,
        }
    }
}

impl KScheme for Serial {
    fn on_irq(&mut self, irq: u8) {
        if irq == self.irq {
            while self.status.read() & 1 == 0 {}

            let mut c = self.data.read() as char;
            let mut sc = 0;

            let console = unsafe { &mut *::env().console.get() };

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
            } else if c == '\x03' {
                console.write(b"^C\n");
                console.commands.send(String::new());

                c = '\0';
                sc = 0;
            } else if c == '\x1B' {
                self.escape = true;
                c = '\0';
            } else if c == '\r' {
                c = '\n';
            } else if c == '\x7F' {
                c = '\0';
                sc = event::K_BKSP;
            }

            if c != '\0' || sc != 0 {
                let key_event = event::KeyEvent {
                    character: c,
                    scancode: sc,
                    pressed: true,
                };

                console.event(key_event.to_event());
            }
        }
    }
}
