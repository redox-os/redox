use spin::Mutex;

use io::{Io, Pio, ReadOnly, WriteOnly};

pub static PS2: Mutex<Ps2> = Mutex::new(Ps2::new());

pub unsafe fn init() {
    PS2.lock().init();
}

bitflags! {
    flags StatusFlags: u8 {
        const OUTPUT_FULL = 1,
        const INPUT_FULL = 1 << 1,
        const SYSTEM = 1 << 2,
        const COMMAND = 1 << 3,
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
        const FIRST_DISABLED = 1 << 4,
        const SECOND_DISABLED = 1 << 5,
        const FIRST_TRANSLATE = 1 << 6,
        // 1 << 7 should be zero
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

#[repr(u8)]
enum KeyboardCommand {
    EnableReporting = 0xF4,
    SetDefaults = 0xF6,
    Reset = 0xFF
}

#[repr(u8)]
enum MouseCommand {
    EnableReporting = 0xF4,
    SetDefaults = 0xF6,
    Reset = 0xFF
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

pub struct Ps2 {
    data: Pio<u8>,
    status: ReadOnly<Pio<u8>>,
    command: WriteOnly<Pio<u8>>,
    mouse: [u8; 3],
    mouse_i: usize
}

impl Ps2 {
    const fn new() -> Ps2 {
        Ps2 {
            data: Pio::new(0x60),
            status: ReadOnly::new(Pio::new(0x64)),
            command: WriteOnly::new(Pio::new(0x64)),
            mouse: [0; 3],
            mouse_i: 0
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
            self.data.read();
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

    fn mouse_command(&mut self, command: MouseCommand) -> u8 {
        self.command(Command::WriteSecond);
        self.write(command as u8);
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
        let self_test = self.read();
        if self_test != 0x55 {
            // TODO: Do reset on failure
            print!("PS/2 Self Test Failure: {:X}\n", self_test);
            return;
        }

        // Enable clocks and interrupts
        {
            let mut config = self.config();
            config.remove(FIRST_DISABLED);
            config.remove(SECOND_DISABLED);
            config.insert(FIRST_INTERRUPT);
            config.insert(SECOND_INTERRUPT);
            self.set_config(config);
        }

        // Enable devices
        self.command(Command::EnableFirst);
        self.command(Command::EnableSecond);

        // Reset and enable scanning on keyboard
        // TODO: Check for ack
        self.keyboard_command(KeyboardCommand::Reset);
        self.keyboard_command(KeyboardCommand::EnableReporting);

        // Reset and enable scanning on mouse
        // TODO: Check for ack
        self.mouse_command(MouseCommand::Reset);
        self.mouse_command(MouseCommand::EnableReporting);

    }

    pub fn on_keyboard(&mut self) {
        let data = self.data.read();
        print!("KEY {:X}\n", data);
    }

    pub fn on_mouse(&mut self) {
        self.mouse[self.mouse_i] = self.data.read();
        self.mouse_i += 1;
        if self.mouse_i >= self.mouse.len() {
            self.mouse_i = 0;

            let flags = MousePacketFlags::from_bits_truncate(self.mouse[0]);

            let mut x = self.mouse[1] as isize;
            if flags.contains(X_SIGN) {
                x -= 0x100;
            }

            let mut y = self.mouse[2] as isize;
            if flags.contains(Y_SIGN) {
                y -= 0x100;
            }

            print!("MOUSE {}, {}, {:?}\n", x, y, flags);
        }
    }
}
