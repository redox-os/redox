use common::pio::*;

pub struct KeyEvent {
	pub character: char,
	pub scancode: u8,
	pub pressed: bool
}

pub struct KeyboardStatus {
	pub lshift: bool,
	pub rshift: bool,
	pub caps_lock: bool,
	pub caps_lock_toggle: bool
}

impl KeyboardStatus {
	fn evaluate(&mut self, scancode_packed: u8){
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
			}else{
				self.caps_lock_toggle = false;
			}
		} else if scancode_packed == 0xBA {
			if self.caps_lock && !self.caps_lock_toggle {
				self.caps_lock = false;
			}
		}
	}

	fn is_shifted(&self) -> bool{
		self.lshift || self.rshift || self.caps_lock
	}
}

pub static mut keyboard_status: KeyboardStatus = KeyboardStatus {
    lshift: false,
    rshift: false,
    caps_lock: false,
    caps_lock_toggle: false
};

pub fn keyboard_interrupt() -> KeyEvent{
	let scancode_packed;
    unsafe{
        scancode_packed = inb(0x60);
    }

	let pressed;
	if scancode_packed < 0x80 {
		pressed = true;
	}else{
		pressed = false;
	}

	let shift;

	unsafe{
		keyboard_status.evaluate(scancode_packed);

		shift = keyboard_status.is_shifted();
	}

	let scancode = scancode_packed & 0x7F;

	KeyEvent { character:char_for_scancode(scancode, shift), scancode:scancode, pressed:pressed }
}

fn char_for_scancode(scancode: u8, shift: bool) -> char{
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

const SCANCODES: [[char; 2]; 58]= [
	['\x00','\x00'],
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
	['\x08','\x08'],
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
	['\x00','\x00'],
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
	['\x00','\x00'],
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
	['\x00','\x00'],
	['\x00','\x00'],
	['\x00','\x00'],
	[' ',' ']
];