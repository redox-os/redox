use alloc::boxed::Box;

use collections::string::String;

use common::event;

use drivers::io::{Io, Pio};

use fs::KScheme;

#[derive(Copy, Clone, Debug, Default)]
#[repr(packed)]
pub struct SerialInfo {
    pub ports: [u16; 4],
}

pub static mut SERIALINFO: Option<SerialInfo> = None;

pub unsafe fn bda_init() {
    SERIALINFO = Some(*(0x400 as *const SerialInfo));
}

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

    pub fn writeb(&mut self, byte: u8){
        while !self.status.readf(0x20) {}
        self.data.write(byte);
    }

    pub fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes.iter() {
            self.writeb(byte);
        }
    }

    pub fn readb(&mut self) -> u8 {
        while self.status.read() & 1 == 0 {}
        self.data.read()
    }
}

impl KScheme for Serial {
    fn on_irq(&mut self, irq: u8) {
        if irq == self.irq {
            let mut c = self.readb() as char;
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
                console.commands.send(String::new(), "Serial Control C");

                if let Some(ref mut inner) = console.inner {
                    inner.redraw = true;
                }

                c = '\0';
                sc = 0;
            } else if c == '\x04' {
                console.write(b"^D\n");

                {
                    let contexts = unsafe { &mut *::env().contexts.get() };
                    debugln!("Magic CTRL-D {}", ::common::time::Duration::monotonic().secs);
                    for context in contexts.iter() {
                        debugln!("  PID {}: {}", context.pid, context.name);

                        if context.blocked > 0 {
                            debugln!("    BLOCKED {}", context.blocked);
                        }

                        if let Some(current_syscall) = context.current_syscall {
                            debugln!("    SYS {:X}: {} {} {:X} {:X} {:X}", current_syscall.0, current_syscall.1, ::syscall::name(current_syscall.1), current_syscall.2, current_syscall.3, current_syscall.4);
                        }
                    }
                }

                if let Some(ref mut inner) = console.inner {
                    inner.redraw = true;
                }

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
