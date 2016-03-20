use alloc::boxed::Box;

use core::cmp;

use common::event::{KeyEvent, MouseEvent};

use drivers::io::{Io, Pio, ReadOnly, WriteOnly};

use graphics::display::VBEMODEINFO;

use fs::KScheme;

use drivers::kb_layouts::layouts;

pub struct Ps2Keyboard<'a> {
    bus: &'a mut Ps2
}

impl<'a> Ps2Keyboard<'a> {
    //TODO: Use result
    fn cmd(&mut self, command: u8) -> u8 {
        self.bus.wait_write();
        self.bus.data.write(command);
        self.bus.wait_read();
        self.bus.data.read()
    }
}

pub struct Ps2Mouse<'a> {
    bus: &'a mut Ps2
}

impl<'a> Ps2Mouse<'a> {
    //TODO: Use result
    fn cmd(&mut self, command: u8) -> u8 {
        self.bus.write(0xD4, command);
        self.bus.wait_read();
        self.bus.data.read()
    }
}

/// PS2
pub struct Ps2 {
    /// The data register
    data: Pio<u8>,
    /// The status register
    sts: ReadOnly<u8, Pio<u8>>,
    /// The command register
    cmd: WriteOnly<u8, Pio<u8>>,
    /// Left shift?
    lshift: bool,
    /// Right shift?
    rshift: bool,
    /// Caps lock?
    caps_lock: bool,
    /// Caps lock toggle
    caps_lock_toggle: bool,
    /// AltGr?
    altgr: bool,
    /// The mouse packet
    mouse_packet: [u8; 4],
    /// Mouse packet index
    mouse_i: usize,
    /// Mouse point x
    mouse_x: i32,
    /// Mouse point y
    mouse_y: i32,
    /// Layout for keyboard
    /// Default: English
    layout: layouts::Layout,
}

impl Ps2 {
    /// Create new PS2 data
    pub fn new() -> Box<Self> {
        let mut module = box Ps2 {
            data: Pio::new(0x60),
            sts: ReadOnly::new(Pio::new(0x64)),
            cmd: WriteOnly::new(Pio::new(0x64)),
            lshift: false,
            rshift: false,
            caps_lock: false,
            caps_lock_toggle: false,
            altgr: false,
            mouse_packet: [0; 4],
            mouse_i: 0,
            mouse_x: 0,
            mouse_y: 0,
            layout: layouts::Layout::English,
        };

        module.init();

        module
    }

    fn wait_read(&self) {
        while self.sts.read() & 1 == 0 {}
    }

    fn wait_write(&self) {
        while self.sts.read() & 2 == 2 {}
    }

    fn cmd(&mut self, command: u8) {
        self.wait_write();
        self.cmd.write(command);
    }

    fn read(&mut self, command: u8) -> u8 {
        self.cmd(command);
        self.wait_read();
        self.data.read()
    }

    fn write(&mut self, command: u8, data: u8) {
        self.cmd(command);
        self.wait_write();
        self.data.write(data);
    }

    fn keyboard<'a>(&'a mut self) -> Ps2Keyboard<'a> {
        Ps2Keyboard {
            bus: self
        }
    }

    fn mouse<'a>(&'a mut self) -> Ps2Mouse<'a> {
        Ps2Mouse {
            bus: self
        }
    }

    fn init(&mut self) {
        while (self.sts.read() & 0x1) == 1 {
            self.data.read();
        }

        // No interrupts, system flag set, clocks enabled, translation disabled
        self.write(0x60, 0b00000100);

        // Enable First Port
        self.cmd(0xAE);
        {
            let mut keyboard = self.keyboard();

            // Reset
            keyboard.cmd(0xFF);

            // Set defaults
            keyboard.cmd(0xF6);

            // Set LEDS
            keyboard.cmd(0xED);
            keyboard.cmd(0);

            // Set Scancode Map:
            keyboard.cmd(0xF0);
            keyboard.cmd(1);

            // Enable Streaming
            keyboard.cmd(0xF4);
        }

        // Enable Second Port
        self.cmd(0xA8);
        {
            let mut mouse = self.mouse();

            // Reset
            mouse.cmd(0xFF);

            // Set defaults
            mouse.cmd(0xF6);

            // Enable Streaming
            mouse.cmd(0xF4);
        }

        // Both interrupts, system flag set, clocks enabled, translation disabled
        self.write(0x60, 0b00000111);

        while (self.sts.read() & 0x1) == 1 {
            debugln!("Extra: {:X}", self.data.read());
        }
    }

    /// Keyboard interrupt
    pub fn keyboard_interrupt(&mut self, mut scancode: u8) -> Option<KeyEvent> {
        if scancode == 0 {
            return None;
        } else if scancode == 0x2A {
            self.lshift = true;
        } else if scancode == 0xAA {
            self.lshift = false;
        } else if scancode == 0x36 {
            self.rshift = true;
        } else if scancode == 0xB6 {
            self.rshift = false;
        } else if scancode == 0x3A {
            if !self.caps_lock {
                self.caps_lock = true;
                self.caps_lock_toggle = true;
            } else {
                self.caps_lock_toggle = false;
            }
        } else if scancode == 0xBA {
            if self.caps_lock && !self.caps_lock_toggle {
                self.caps_lock = false;
            }
        } else if scancode == 0xE0 {
            let scancode_byte_2 = self.data.read();
            if scancode_byte_2 == 0x38 {
                self.altgr = true;
            } else if scancode_byte_2 == 0xB8 {
                self.altgr = false;
            } else {
                scancode = scancode_byte_2;
            }
        }

        let shift = self.caps_lock != (self.lshift || self.rshift);

        return Some(KeyEvent {
            character: layouts::char_for_scancode(scancode & 0x7F, shift, self.altgr, &self.layout),
            scancode: scancode & 0x7F,
            pressed: scancode < 0x80,
        });
    }

    /// Mouse interrupt
    pub fn mouse_interrupt(&mut self, byte: u8) -> Option<MouseEvent> {
        if self.mouse_i == 0 {
            if byte & 0x8 == 0x8 {
                self.mouse_packet[0] = byte;
                self.mouse_i += 1;
            }
        } else if self.mouse_i == 1 {
            self.mouse_packet[1] = byte;

            self.mouse_i += 1;
        } else {
            self.mouse_packet[2] = byte;

            let left_button = (self.mouse_packet[0] & 1) == 1;
            let right_button = (self.mouse_packet[0] & 2) == 2;
            let middle_button = (self.mouse_packet[0] & 4) == 4;

            let x;
            if (self.mouse_packet[0] & 0x40) != 0x40 && self.mouse_packet[1] != 0 {
                x = (self.mouse_packet[1] as isize -
                     (((self.mouse_packet[0] as isize) << 4) & 0x100)) as i32;
            } else {
                x = 0;
            }

            let y;
            if (self.mouse_packet[0] & 0x80) != 0x80 && self.mouse_packet[2] != 0 {
                y = ((((self.mouse_packet[0] as isize) << 3) & 0x100) -
                     self.mouse_packet[2] as isize) as i32;
            } else {
                y = 0;
            }

            if let Some(mode_info) = unsafe { VBEMODEINFO } {
                self.mouse_x = cmp::max(0, cmp::min(mode_info.xresolution as i32, self.mouse_x + x));
                self.mouse_y = cmp::max(0, cmp::min(mode_info.yresolution as i32, self.mouse_y + y));
            }

            self.mouse_i = 0;

            return Some(MouseEvent {
                x: self.mouse_x,
                y: self.mouse_y,
                left_button: left_button,
                right_button: right_button,
                middle_button: middle_button,
            });
        }

        return None;
    }

    /// Function to change the layout of the keyboard
    pub fn change_layout(&mut self, layout: usize) {
        self.layout = match layout {
            0 => layouts::Layout::English,
            1 => layouts::Layout::French,
            2 => layouts::Layout::German,
            _ => layouts::Layout::English,
        }
    }
}

impl KScheme for Ps2 {
    fn on_irq(&mut self, irq: u8) {
        if irq == 0xC {
            let data = self.data.read();
            if let Some(mouse_event) = self.mouse_interrupt(data) {
                if ::env().console.lock().draw {
                    //Ignore mouse event
                } else {
                    ::env().events.send(mouse_event.to_event());
                }
            }
        } else if irq == 0x1 {
            let data = self.data.read();
            if let Some(key_event) = self.keyboard_interrupt(data) {
                if ::env().console.lock().draw {
                    ::env().console.lock().event(key_event.to_event());
                } else {
                    ::env().events.send(key_event.to_event());
                }
            }
        }
    }
}
