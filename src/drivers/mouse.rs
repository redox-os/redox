use common::pio::*;

pub const MOUSE_CURSOR: [u8; 16] = [
    0b10000000,
    0b11000000,
    0b11100000,
    0b11110000,
    0b11111000,
    0b11111100,
    0b11111110,
    0b11111111,
    0b11100000,
    0b11000000,
    0b10000000,
    0b00000000,
    0b00000000,
    0b00000000,
    0b00000000,
    0b00000000
];

#[derive(Copy, Clone)]
pub struct MouseEvent {
	pub x: isize,
	pub y: isize,
	pub left_button: bool,
	pub right_button: bool,
	pub middle_button: bool,
	pub valid: bool
}

static mut mouse_cycle: usize = 0;
static mut mouse_byte: [u8; 3] = [0, 0, 0];

pub unsafe fn mouse_wait0(){
    while (inb(0x64) & 1) == 0 {
    }
}

pub unsafe fn mouse_wait1(){
    while (inb(0x64) & 2) == 2 {
    }
}

pub unsafe fn mouse_init(){
    mouse_cycle = 0;
    mouse_byte = [0, 0, 0];

    mouse_wait1();
    outb(0x64, 0xA8);

	mouse_wait1();
	outb(0x64, 0x20);
	mouse_wait0();
	let status = inb(0x60) | 2;
	mouse_wait1();
	outb(0x64, 0x60);
	mouse_wait1();
	outb(0x60, status);

	mouse_wait1();
	outb(0x64, 0xD4);
	mouse_wait1();
	outb(0x60, 0xF6);
	mouse_wait0();
	inb(0x60);

	mouse_wait1();
	outb(0x64, 0xD4);
	mouse_wait1();
	outb(0x60, 0xF4);
	mouse_wait0();
	inb(0x60);
}

pub fn mouse_interrupt() -> MouseEvent {
    unsafe{
        let mut x = 0;
        let mut y = 0;
        let mut left_button = false;
        let mut right_button = false;
        let mut middle_button = false;
        let mut valid = false;

        let packet = inb(0x60);
        if mouse_cycle == 0 {
            if packet & 0x8 == 0x8 {
                mouse_byte[0] = packet;
                mouse_cycle += 1;
            }
        } else if mouse_cycle == 1 {
            mouse_byte[1] = packet;

            mouse_cycle += 1;
        } else {
            mouse_byte[2] = packet;

            if (mouse_byte[0] & 1) == 1 {
                left_button = true;
            }
            if (mouse_byte[0] & 2) == 2 {
                right_button = true;
            }
            if (mouse_byte[0] & 4) == 4 {
                middle_button = true;
            }

            if (mouse_byte[0] & 0x40) != 0x40 && mouse_byte[1] != 0 {
                x += mouse_byte[1] as isize - (((mouse_byte[0] as isize) << 4) & 0x100);
            }

            if (mouse_byte[0] & 0x80) != 0x80 && mouse_byte[2] != 0 {
                y += (((mouse_byte[0] as isize) << 3) & 0x100) - mouse_byte[2] as isize;
            }

            valid = true;

            mouse_cycle = 0;
        }

        MouseEvent{ x:x/4, y:y/4, left_button:left_button, right_button:right_button, middle_button:middle_button, valid:valid }
    }
}