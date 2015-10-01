use common::debug::*;
use common::event::*;

use drivers::pio::*;

pub struct KeyboardStatus {
    pub lshift: bool,
    pub rshift: bool,
    pub caps_lock: bool,
    pub caps_lock_toggle: bool
}

impl KeyboardStatus {
    fn evaluate(&mut self, scancode_packed: u8) {
        if scancode_packed == 0x2A {
            self.lshift = true;
        } else if scancode_packed == 0xAA {
            self.lshift = false;
        } else if scancode_packed == 0x36 {
            self.rshift = true;
        } else if scancode_packed == 0xB6 {
            self.rshift = false;
        } else if scancode_packed == 0x3A {
            if !self.caps_lock {
                self.caps_lock = true;
                self.caps_lock_toggle = true;
            } else {
                self.caps_lock_toggle = false;
            }
        } else if scancode_packed == 0xBA {
            if self.caps_lock && !self.caps_lock_toggle {
                self.caps_lock = false;
            }
        }
    }

    fn is_shifted(&self) -> bool {
        if self.caps_lock {
            !(self.lshift || self.rshift)
        } else {
            (self.lshift || self.rshift)
        }
    }
}

pub static mut keyboard_status: KeyboardStatus = KeyboardStatus {
    lshift: false,
    rshift: false,
    caps_lock: false,
    caps_lock_toggle: false
};

unsafe fn keyboard_wait0() {
    while (inb(0x64) & 1) == 0 {}
}

unsafe fn keyboard_wait1() {
    while (inb(0x64) & 2) == 2 {}
}

pub unsafe fn keyboard_init() {
    keyboard_status.lshift = false;
    keyboard_status.rshift = false;
    keyboard_status.caps_lock = false;
    keyboard_status.caps_lock_toggle = false;

    d("Clear buffer:");
    while (inb(0x64) & 0x1) == 1 {
        dc(' ');
        dbh(inb(0x60));
    }
    dl();

    d("Enable Keyboard:");
        keyboard_wait1();
        outb(0x64, 0x20);
        keyboard_wait0();
        let mut status = inb(0x60);
        dc(' ');
        dbh(status);
        status = (status & 0b00110111) | 1 | 0b10000;
        dc(' ');
        dbh(status);
        keyboard_wait1();
        outb(0x64, 0x60);
        keyboard_wait1();
        outb(0x60, status);
    dl();

    d("Set Defaults:");
        keyboard_wait1();
        outb(0x60, 0xF6);
        keyboard_wait0();
        dc(' ');
        dbh(inb(0x60));
    dl();

    d("Set LEDS:");
        keyboard_wait1();
        outb(0x60, 0xED);
        keyboard_wait0();
        dc(' ');
        dbh(inb(0x60));

        keyboard_wait1();
        outb(0x60, 0);
        keyboard_wait0();
        dc(' ');
        dbh(inb(0x60));
    dl();

    d("Set Scancode Map:");
        keyboard_wait1();
        outb(0x60, 0xF0);
        keyboard_wait0();
        dc(' ');
        dbh(inb(0x60));

        keyboard_wait1();
        outb(0x60, 1);
        keyboard_wait0();
        dc(' ');
        dbh(inb(0x60));
    dl();
}

pub fn keyboard_interrupt() -> KeyEvent {
    let scancode_packed;
    unsafe {
        scancode_packed = inb(0x60);
    }

    let pressed;
    if scancode_packed < 0x80 {
        pressed = true;
    } else {
        pressed = false;
    }

    let shift;

    unsafe {
        keyboard_status.evaluate(scancode_packed);

        shift = keyboard_status.is_shifted();
    }

    let scancode = scancode_packed & 0x7F;

    KeyEvent { character:char_for_scancode(scancode, shift), scancode:scancode, pressed:pressed }
}

fn char_for_scancode(scancode: u8, shift: bool) -> char {
	let mut character = '\x00';
	if scancode < 58 {
        if shift {
            character = SCANCODES[scancode as usize][1];
        }else{
            character = SCANCODES[scancode as usize][0];
        }
	}
	character
}

static SCANCODES: [[char; 2]; 58]= [
    ['\0','\0'],
    ['\x1B','\x1B'],
    ['1','!'],
    ['2','@'],
    ['3','#'],
    ['4','$'],
    ['5','%'],
    ['6','^'],
    ['7','&'],
    ['8','*'],
    ['9','('],
    ['0',')'],
    ['-','_'],
    ['=','+'],
    ['\0','\0'],
    ['\t','\t'],
    ['q','Q'],
    ['w','W'],
    ['e','E'],
    ['r','R'],
    ['t','T'],
    ['y','Y'],
    ['u','U'],
    ['i','I'],
    ['o','O'],
    ['p','P'],
    ['[','{'],
    [']','}'],
    ['\n','\n'],
    ['\0','\0'],
    ['a','A'],
    ['s','S'],
    ['d','D'],
    ['f','F'],
    ['g','G'],
    ['h','H'],
    ['j','J'],
    ['k','K'],
    ['l','L'],
    [';',':'],
    ['\'','"'],
    ['`','~'],
    ['\0','\0'],
    ['\\','|'],
    ['z','Z'],
    ['x','X'],
    ['c','C'],
    ['v','V'],
    ['b','B'],
    ['n','N'],
    ['m','M'],
    [',','<'],
    ['.','>'],
    ['/','?'],
    ['\0','\0'],
    ['\0','\0'],
    ['\0','\0'],
    [' ',' ']
];
