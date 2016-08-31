use spin::Mutex;

use io::{Io, Pio, ReadOnly, WriteOnly};

pub static PS2: Mutex<Ps2> = Mutex::new(Ps2::new());

pub unsafe fn init() {
    PS2.lock().init();
}

mod status {
    bitflags! {
        pub flags Flags: u8 {
            const OUTPUT_FULL = 1,
            const INPUT_FULL = 1 << 1,
            const SYSTEM = 1 << 2,
            const COMMAND = 1 << 3,
            const TIME_OUT = 1 << 6,
            const PARITY = 1 << 7
        }
    }
}

mod config {
    bitflags! {
        pub flags Flags: u8 {
            const FIRST_INTERRUPT = 1,
            const SECOND_INTERRUPT = 1 << 1,
            const SYSTEM = 1 << 2,
            const FIRST_DISABLE = 1 << 4,
            const SECOND_DISABLE = 1 << 5,
            const FIRST_TRANSLATE = 1 << 6
        }
    }
}

#[repr(u8)]
enum Command {
    ReadConfig = 0x20,
    WriteConfig = 0x60,
    DisableSecond = 0xA7,
    EnableSecond = 0xA8,
    TestSecond = 0xA9,
    TestController = 0xAA,
    TestFirst = 0xAB,
    Diagnostic = 0xAC,
    DisableFirst = 0xAD,
    EnableFirst = 0xAE,
    WriteSecond = 0xD4
}

pub struct Ps2 {
    pub data: Pio<u8>,
    pub status: ReadOnly<Pio<u8>>,
    pub command: WriteOnly<Pio<u8>>
}

impl Ps2 {
    pub const fn new() -> Ps2 {
        Ps2 {
            data: Pio::new(0x60),
            status: ReadOnly::new(Pio::new(0x64)),
            command: WriteOnly::new(Pio::new(0x64))
        }
    }

    pub fn init(&mut self) {
        print!("Status {:?}\n", status::Flags::from_bits_truncate(self.status.read()));

        self.command.write(Command::ReadConfig as u8);
        print!("Config {:?}\n", config::Flags::from_bits_truncate(self.data.read()));
    }
}
