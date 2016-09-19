use core::cmp;

use io::{Io, Pio, ReadOnly, WriteOnly};

pub unsafe fn init() {
    Ps2::new().init();
}

bitflags! {
    flags StatusFlags: u8 {
        const OUTPUT_FULL = 1,
        const INPUT_FULL = 1 << 1,
        const SYSTEM = 1 << 2,
        const COMMAND = 1 << 3,
        // Chipset specific
        const KEYBOARD_LOCK = 1 << 4,
        // Chipset specific
        const SECOND_OUTPUT_FULL = 1 << 5,
        const TIME_OUT = 1 << 6,
        const PARITY = 1 << 7
    }
}

bitflags! {
    flags ConfigFlags: u8 {
        const FIRST_INTERRUPT = 1,
        const SECOND_INTERRUPT = 1 << 1,
        const POST_PASSED = 1 << 2,
        // 1 << 3 should be zero
        const CONFIG_RESERVED_3 = 1 << 3,
        const FIRST_DISABLED = 1 << 4,
        const SECOND_DISABLED = 1 << 5,
        const FIRST_TRANSLATE = 1 << 6,
        // 1 << 7 should be zero
        const CONFIG_RESERVED_7 = 1 << 7,
    }
}

#[repr(u8)]
#[allow(dead_code)]
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

#[repr(u8)]
#[allow(dead_code)]
enum KeyboardCommand {
    EnableReporting = 0xF4,
    SetDefaults = 0xF6,
    Reset = 0xFF
}

#[repr(u8)]
enum KeyboardCommandData {
    ScancodeSet = 0xF0
}

#[repr(u8)]
#[allow(dead_code)]
enum MouseCommand {
    GetDeviceId = 0xF2,
    EnableReporting = 0xF4,
    SetDefaults = 0xF6,
    Reset = 0xFF
}

#[repr(u8)]
enum MouseCommandData {
    SetSampleRate = 0xF3,
}

bitflags! {
    flags MousePacketFlags: u8 {
        const LEFT_BUTTON = 1,
        const RIGHT_BUTTON = 1 << 1,
        const MIDDLE_BUTTON = 1 << 2,
        const ALWAYS_ON = 1 << 3,
        const X_SIGN = 1 << 4,
        const Y_SIGN = 1 << 5,
        const X_OVERFLOW = 1 << 6,
        const Y_OVERFLOW = 1 << 7
    }
}

pub struct Ps2Mouse {
    data: ReadOnly<Pio<u8>>,
    mouse: [u8; 4],
    mouse_i: usize,
    mouse_extra: bool,
    mouse_x: usize,
    mouse_y: usize
}

impl Ps2Mouse {
    fn new(mouse_extra: bool) -> Self {
        Ps2Mouse {
            data: ReadOnly::new(Pio::new(0x60)),
            mouse: [0; 4],
            mouse_i: 0,
            mouse_extra: mouse_extra,
            mouse_x: 0,
            mouse_y: 0
        }
    }

    pub fn on_irq(&mut self) {
        self.mouse[self.mouse_i] = self.data.read();
        self.mouse_i += 1;

        let flags = MousePacketFlags::from_bits_truncate(self.mouse[0]);
        if ! flags.contains(ALWAYS_ON) {
            println!("MOUSE MISALIGN {:X}", self.mouse[0]);

            self.mouse = [0; 4];
            self.mouse_i = 0;
        } else if self.mouse_i >= self.mouse.len() || (!self.mouse_extra && self.mouse_i >= 3) {
            if ! flags.contains(X_OVERFLOW) && ! flags.contains(Y_OVERFLOW) {
                let mut dx = self.mouse[1] as isize;
                if flags.contains(X_SIGN) {
                    dx -= 0x100;
                }

                let mut dy = self.mouse[2] as isize;
                if flags.contains(Y_SIGN) {
                    dy -= 0x100;
                }

                let _extra = if self.mouse_extra {
                    self.mouse[3]
                } else {
                    0
                };

                //print!("MOUSE {:?}, {}, {}, {}\n", flags, dx, dy, extra);

                if let Some(ref mut display) = *super::display::DISPLAY.lock() {
                    self.mouse_x = cmp::max(0, cmp::min(display.width as isize - 1, self.mouse_x as isize + dx)) as usize;
                    self.mouse_y = cmp::max(0, cmp::min(display.height as isize - 1, self.mouse_y as isize - dy)) as usize;
                    let offset = self.mouse_y * display.width + self.mouse_x;
                    display.onscreen[offset as usize] = 0xFF0000;
                }
            } else {
                println!("MOUSE OVERFLOW {:X} {:X} {:X} {:X}", self.mouse[0], self.mouse[1], self.mouse[2], self.mouse[3]);
            }

            self.mouse = [0; 4];
            self.mouse_i = 0;
        }
    }
}

pub struct Ps2 {
    data: Pio<u8>,
    status: ReadOnly<Pio<u8>>,
    command: WriteOnly<Pio<u8>>
}

impl Ps2 {
    const fn new() -> Self {
        Ps2 {
            data: Pio::new(0x60),
            status: ReadOnly::new(Pio::new(0x64)),
            command: WriteOnly::new(Pio::new(0x64)),
        }
    }

    fn status(&mut self) -> StatusFlags {
        StatusFlags::from_bits_truncate(self.status.read())
    }

    fn wait_write(&mut self) {
        while self.status().contains(INPUT_FULL) {}
    }

    fn wait_read(&mut self) {
        while ! self.status().contains(OUTPUT_FULL) {}
    }

    fn flush_read(&mut self) {
        while self.status().contains(OUTPUT_FULL) {
            print!("FLUSH: {:X}\n", self.data.read());
        }
    }

    fn command(&mut self, command: Command) {
        self.wait_write();
        self.command.write(command as u8);
    }

    fn read(&mut self) -> u8 {
        self.wait_read();
        self.data.read()
    }

    fn write(&mut self, data: u8) {
        self.wait_write();
        self.data.write(data);
    }

    fn config(&mut self) -> ConfigFlags {
        self.command(Command::ReadConfig);
        ConfigFlags::from_bits_truncate(self.read())
    }

    fn set_config(&mut self, config: ConfigFlags) {
        self.command(Command::WriteConfig);
        self.write(config.bits());
    }

    fn keyboard_command(&mut self, command: KeyboardCommand) -> u8 {
        self.write(command as u8);
        self.read()
    }

    fn keyboard_command_data(&mut self, command: KeyboardCommandData, data: u8) -> u8 {
        self.write(command as u8);
        assert_eq!(self.read(), 0xFA);
        self.write(data as u8);
        self.read()
    }

    fn mouse_command(&mut self, command: MouseCommand) -> u8 {
        self.command(Command::WriteSecond);
        self.write(command as u8);
        self.read()
    }

    fn mouse_command_data(&mut self, command: MouseCommandData, data: u8) -> u8 {
        self.command(Command::WriteSecond);
        self.write(command as u8);
        assert_eq!(self.read(), 0xFA);
        self.command(Command::WriteSecond);
        self.write(data as u8);
        self.read()
    }

    fn init(&mut self) {
        // Disable devices
        self.command(Command::DisableFirst);
        self.command(Command::DisableSecond);

        // Clear remaining data
        self.flush_read();

        // Disable clocks, disable interrupts, and disable translate
        {
            let mut config = self.config();
            config.insert(FIRST_DISABLED);
            config.insert(SECOND_DISABLED);
            config.remove(FIRST_TRANSLATE);
            config.remove(FIRST_INTERRUPT);
            config.remove(SECOND_INTERRUPT);
            self.set_config(config);
        }

        // Perform the self test
        self.command(Command::TestController);
        assert_eq!(self.read(), 0x55);

        // Enable devices
        self.command(Command::EnableFirst);
        self.command(Command::EnableSecond);

        // Reset keyboard
        assert_eq!(self.keyboard_command(KeyboardCommand::Reset), 0xFA);
        assert_eq!(self.read(), 0xAA);
        self.flush_read();

        // Set scancode set to 2
        assert_eq!(self.keyboard_command_data(KeyboardCommandData::ScancodeSet, 2), 0xFA);

        // Reset mouse and set up scroll
        // TODO: Check for ack
        assert_eq!(self.mouse_command(MouseCommand::Reset), 0xFA);
        assert_eq!(self.read(), 0xAA);
        assert_eq!(self.read(), 0x00);
        self.flush_read();

        // Enable extra packet on mouse
        assert_eq!(self.mouse_command_data(MouseCommandData::SetSampleRate, 200), 0xFA);
        assert_eq!(self.mouse_command_data(MouseCommandData::SetSampleRate, 100), 0xFA);
        assert_eq!(self.mouse_command_data(MouseCommandData::SetSampleRate, 80), 0xFA);
        assert_eq!(self.mouse_command(MouseCommand::GetDeviceId), 0xFA);
        let mouse_id = self.read();
        let mouse_extra = mouse_id == 3;

        // Enable extra buttons, TODO
        /*
        if self.mouse_extra {
            print!("SAMPLE 200 {:X}\n", self.mouse_command_data(MouseCommandData::SetSampleRate, 200));
            print!("SAMPLE 200 {:X}\n", self.mouse_command_data(MouseCommandData::SetSampleRate, 200));
            print!("SAMPLE 80 {:X}\n", self.mouse_command_data(MouseCommandData::SetSampleRate, 80));
            print!("GET ID {:X}\n", self.mouse_command(MouseCommand::GetDeviceId));
            let mouse_id = self.read();
            print!("MOUSE ID: {:X} == 0x04\n", mouse_id);
        }
        */

        // Set sample rate to maximum
        assert_eq!(self.mouse_command_data(MouseCommandData::SetSampleRate, 200), 0xFA);

        // Enable data reporting
        assert_eq!(self.keyboard_command(KeyboardCommand::EnableReporting), 0xFA);
        assert_eq!(self.mouse_command(MouseCommand::EnableReporting), 0xFA);

        // Enable clocks and interrupts
        {
            let mut config = self.config();
            config.remove(FIRST_DISABLED);
            config.remove(SECOND_DISABLED);
            config.insert(FIRST_TRANSLATE);
            config.insert(FIRST_INTERRUPT);
            config.insert(SECOND_INTERRUPT);
            self.set_config(config);
        }

        self.flush_read();
    }
}
