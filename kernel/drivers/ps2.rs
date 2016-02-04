use alloc::boxed::Box;

use core::cmp;

use common::event::{KeyEvent, MouseEvent};

use drivers::io::{Io, Pio};

use graphics::display::VBEMODEINFO;

use fs::KScheme;

use drivers::kb_layouts::layouts;

/// PS2
pub struct Ps2 {
    /// The data
    data: Pio<u8>,
    /// The command
    cmd: Pio<u8>,
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
            data: Pio::<u8>::new(0x60),
            cmd: Pio::<u8>::new(0x64),
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

        unsafe {
            module.keyboard_init();
            module.mouse_init();
        }

        module
    }

    unsafe fn wait0(&self) {
        while (self.cmd.read() & 1) == 0 {}
    }

    unsafe fn wait1(&self) {
        while (self.cmd.read() & 2) == 2 {}
    }

    unsafe fn keyboard_init(&mut self) {
        while (self.cmd.read() & 0x1) == 1 {
            self.data.read();
        }

        self.wait1();
        self.cmd.write(0x20);
        self.wait0();
        let flags = (self.data.read() & 0b00110111) | 1 | 0b10000;
        self.wait1();
        self.cmd.write(0x60);
        self.wait1();
        self.data.write(flags);

        // Set Defaults
        self.wait1();
        self.data.write(0xF6);
        self.wait0();
        self.data.read();

        // Set LEDS
        self.wait1();
        self.data.write(0xED);
        self.wait0();
        self.data.read();

        self.wait1();
        self.data.write(0);
        self.wait0();
        self.data.read();

        // Set Scancode Map:
        self.wait1();
        self.data.write(0xF0);
        self.wait0();
        self.data.read();

        self.wait1();
        self.data.write(1);
        self.wait0();
        self.data.read();
    }

    /// Keyboard interrupt
    pub fn keyboard_interrupt(&mut self) -> Option<KeyEvent> {
        let mut scancode = self.data.read();

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

    unsafe fn mouse_cmd(&mut self, byte: u8) -> u8 {
        self.wait1();
        self.cmd.write(0xD4);
        self.wait1();
        self.data.write(byte);

        self.wait0();
        self.data.read()
    }

    /// Initialize mouse
    pub unsafe fn mouse_init(&mut self) {
        // The Init Dance
        self.wait1();
        self.cmd.write(0xA8);

        self.wait1();
        self.cmd.write(0x20);
        self.wait0();
        let status = self.data.read() | 2;
        self.wait1();
        self.cmd.write(0x60);
        self.wait1();
        self.data.write(status);

        // Set defaults
        self.mouse_cmd(0xF6);

        // Enable Streaming
        self.mouse_cmd(0xF4);
    }

    /// Mouse interrupt
    pub fn mouse_interrupt(&mut self) -> Option<MouseEvent> {
        let byte = self.data.read();
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

            unsafe {
                let mode_info = &*VBEMODEINFO;
                self.mouse_x = cmp::max(0,
                                        cmp::min(mode_info.xresolution as i32, self.mouse_x + x));
                self.mouse_y = cmp::max(0,
                                        cmp::min(mode_info.yresolution as i32, self.mouse_y + y));
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
        if irq == 0x1 || irq == 0xC {
            self.on_poll();
        }
    }

    fn on_poll(&mut self) {
        loop {
            let status = self.cmd.read();
            if status & 0x21 == 1 {
                if let Some(key_event) = self.keyboard_interrupt() {
                    ::env().events.lock().push_back(key_event.to_event());
                }
            } else if status & 0x21 == 0x21 {
                if let Some(mouse_event) = self.mouse_interrupt() {
                    ::env().events.lock().push_back(mouse_event.to_event());
                }
            } else {
                break;
            }
        }
    }
}
