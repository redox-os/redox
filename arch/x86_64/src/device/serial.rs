use core::fmt::{self, Write};
use spin::Mutex;

use io::{Io, Pio, ReadOnly};

pub static COM1: Mutex<SerialPort> = Mutex::new(SerialPort::new(0x3F8));
pub static COM2: Mutex<SerialPort> = Mutex::new(SerialPort::new(0x2F8));

pub unsafe fn init() {
    COM1.lock().init();
    COM2.lock().init();
}

bitflags! {
    /// Interrupt enable flags
    flags IntEnFlags: u8 {
        const RECEIVED = 1,
        const SENT = 1 << 1,
        const ERRORED = 1 << 2,
        const STATUS_CHANGE = 1 << 3,
        // 4 to 7 are unused
    }
}

bitflags! {
    /// Line status flags
    flags LineStsFlags: u8 {
        const INPUT_FULL = 1,
        // 1 to 4 unknown
        const OUTPUT_EMPTY = 1 << 5,
        // 6 and 7 unknown
    }
}

#[allow(dead_code)]
pub struct SerialPort {
    /// Data register, read to receive, write to send
    data: Pio<u8>,
    /// Interrupt enable
    int_en: Pio<u8>,
    /// FIFO control
    fifo_ctrl: Pio<u8>,
    /// Line control
    line_ctrl: Pio<u8>,
    /// Modem control
    modem_ctrl: Pio<u8>,
    /// Line status
    line_sts: ReadOnly<Pio<u8>>,
    /// Modem status
    modem_sts: ReadOnly<Pio<u8>>,
}

impl SerialPort {
    const fn new(base: u16) -> SerialPort {
        SerialPort {
            data: Pio::new(base),
            int_en: Pio::new(base + 1),
            fifo_ctrl: Pio::new(base + 2),
            line_ctrl: Pio::new(base + 3),
            modem_ctrl: Pio::new(base + 4),
            line_sts: ReadOnly::new(Pio::new(base + 5)),
            modem_sts: ReadOnly::new(Pio::new(base + 6))
        }
    }

    fn line_sts(&self) -> LineStsFlags {
        LineStsFlags::from_bits_truncate(self.line_sts.read())
    }

    fn write(&mut self, data: u8) {
        while ! self.line_sts().contains(OUTPUT_EMPTY) {}
        self.data.write(data)
    }

    fn write_translate(&mut self, data: u8) {
        match data {
            8 | 0x7F => {
                self.write(8);
                self.write(b' ');
                self.write(8);
            },
            _ => {
                self.write(data);
            }
        }
    }

    fn init(&mut self) {
        //TODO: Cleanup
        self.int_en.write(0x00);
        self.line_ctrl.write(0x80);
        self.data.write(0x03);
        self.int_en.write(0x00);
        self.line_ctrl.write(0x03);
        self.fifo_ctrl.write(0xC7);
        self.modem_ctrl.write(0x0B);
        self.int_en.write(0x01);
    }

    pub fn on_receive(&mut self) {
        let data = self.data.read();

        extern {
            fn debug_input(byte: u8);
        }

        unsafe { debug_input(data) };
    }
}

impl Write for SerialPort {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        for byte in s.bytes() {
            self.write_translate(byte);
        }

        Ok(())
    }
}
