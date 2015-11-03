use alloc::boxed::Box;

use common::event::{KeyEvent, MouseEvent};

use drivers::pio::*;

use schemes::KScheme;

/// PS2
pub struct PS2 {
    /// The data
    data: PIO8,
    /// The command
    cmd: PIO8,
    /// Left shift?
    lshift: bool,
    /// Right shift?
    rshift: bool,
    /// Caps lock?
    caps_lock: bool,
    /// Caps lock toggle
    caps_lock_toggle: bool,
    /// The mouse packet
    mouse_packet: [u8; 4],
    /// Mouse
    mouse_i: usize,
}

impl PS2 {
    /// Create new PS2 data
    pub fn new() -> Box<Self> {
        let mut module = box PS2 {
            data: PIO8::new(0x60),
            cmd: PIO8::new(0x64),
            lshift: false,
            rshift: false,
            caps_lock: false,
            caps_lock_toggle: false,
            mouse_packet: [0; 4],
            mouse_i: 0,
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

        //Set Defaults
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

        //Set Scancode Map:
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
        let scancode = unsafe { self.data.read() };

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
        }

        let shift;
        if self.caps_lock {
            shift = !(self.lshift || self.rshift);
        } else {
            shift = self.lshift || self.rshift;
        }

        return Some(KeyEvent {
            character: char_for_scancode(scancode & 0x7F, shift),
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
        //The Init Dance
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

        //Set defaults
        self.mouse_cmd(0xF6);

        //Enable Streaming
        self.mouse_cmd(0xF4);
    }

    /// Mouse interrupt
    pub fn mouse_interrupt(&mut self) -> Option<MouseEvent> {
        let byte = unsafe { self.data.read() };
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
                x = self.mouse_packet[1] as isize -
                    (((self.mouse_packet[0] as isize) << 4) & 0x100);
            } else {
                x = 0;
            }

            let y;
            if (self.mouse_packet[0] & 0x80) != 0x80 && self.mouse_packet[2] != 0 {
                y = (((self.mouse_packet[0] as isize) << 3) & 0x100) -
                    self.mouse_packet[2] as isize;
            } else {
                y = 0;
            }

            self.mouse_i = 0;

            return Some(MouseEvent {
                x: x,
                y: y,
                left_button: left_button,
                right_button: right_button,
                middle_button: middle_button,
            });
        }

        return None;
    }
}


impl KScheme for PS2 {
    fn on_irq(&mut self, irq: u8) {
        if irq == 0x1 || irq == 0xC {
            self.on_poll();
        }
    }

    fn on_poll(&mut self) {
        loop {
            let status = unsafe { self.cmd.read() };
            if status & 0x21 == 1 {
                if let Some(key_event) = self.keyboard_interrupt() {
                    key_event.trigger();
                }
            } else if status & 0x21 == 0x21 {
                if let Some(mouse_event) = self.mouse_interrupt() {
                    mouse_event.trigger();
                }
            } else {
                break;
            }
        }
    }
}

fn char_for_scancode(scancode: u8, shift: bool) -> char {
    let mut character = '\x00';
    if scancode < 58 {
        if shift {
            character = SCANCODES[scancode as usize][1];
        } else {
            character = SCANCODES[scancode as usize][0];
        }
    }
    character
}

static SCANCODES: [[char; 2]; 58]= [['\0', '\0'],
 ['\x1B', '\x1B'],
 ['1', '!'],
 ['2', '@'],
 ['3', '#'],
 ['4', '$'],
 ['5', '%'],
 ['6', '^'],
 ['7', '&'],
 ['8', '*'],
 ['9', '('],
 ['0', ')'],
 ['-', '_'],
 ['=', '+'],
 ['\0', '\0'],
 ['\t', '\t'],
 ['q', 'Q'],
 ['w', 'W'],
 ['e', 'E'],
 ['r', 'R'],
 ['t', 'T'],
 ['y', 'Y'],
 ['u', 'U'],
 ['i', 'I'],
 ['o', 'O'],
 ['p', 'P'],
 ['[', '{'],
 [']', '}'],
 ['\n', '\n'],
 ['\0', '\0'],
 ['a', 'A'],
 ['s', 'S'],
 ['d', 'D'],
 ['f', 'F'],
 ['g', 'G'],
 ['h', 'H'],
 ['j', 'J'],
 ['k', 'K'],
 ['l', 'L'],
 [';', ':'],
 ['\'', '"'],
 ['`', '~'],
 ['\0', '\0'],
 ['\\', '|'],
 ['z', 'Z'],
 ['x', 'X'],
 ['c', 'C'],
 ['v', 'V'],
 ['b', 'B'],
 ['n', 'N'],
 ['m', 'M'],
 [',', '<'],
 ['.', '>'],
 ['/', '?'],
 ['\0', '\0'],
 ['\0', '\0'],
 ['\0', '\0'],
 [' ', ' ']];
