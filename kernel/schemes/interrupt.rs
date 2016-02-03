use alloc::boxed::Box;

use schemes::{Result, KScheme, Resource, Url, VecResource};

pub struct InterruptScheme;

static IRQ_NAME: [&'static str; 16] = [
    "Programmable Interval Timer",
    "Keyboard",
    "Cascade",
    "Serial 2 and 4",
    "Serial 1 and 3",
    "Parallel 2",
    "Floppy",
    "Parallel 1",
    "Realtime Clock",
    "PCI 1",
    "PCI 2",
    "PCI 3",
    "Mouse",
    "Coprocessor",
    "IDE Primary",
    "IDE Secondary",
];

impl KScheme for InterruptScheme {
    fn scheme(&self) -> &str {
        "interrupt"
    }

    fn open<'a, 'b: 'a>(&'a mut self, _: Url<'b>, _: usize) -> Result<Box<Resource + 'a>> {
        let mut string = format!("{:<6}{:<16}{}", "INT", "COUNT", "DESCRIPTION");

        {
            let interrupts = ::env().interrupts.lock();
            for interrupt in 0..interrupts.len() {
                let count = interrupts[interrupt];

                if count > 0 {
                    let description = match interrupt {
                        i @ 0x20 ... 0x30 => IRQ_NAME[i - 0x20],
                        0x80 => "System Call",
                        0x0 => "Divide by zero exception",
                        0x1 => "Debug exception",
                        0x2 => "Non-maskable interrupt",
                        0x3 => "Breakpoint exception",
                        0x4 => "Overflow exception",
                        0x5 => "Bound range exceeded exception",
                        0x6 => "Invalid opcode exception",
                        0x7 => "Device not available exception",
                        0x8 => "Double fault",
                        0xA => "Invalid TSS exception",
                        0xB => "Segment not present exception",
                        0xC => "Stack-segment fault",
                        0xD => "General protection fault",
                        0xE => "Page fault",
                        0x10 => "x87 floating-point exception",
                        0x11 => "Alignment check exception",
                        0x12 => "Machine check exception",
                        0x13 => "SIMD floating-point exception",
                        0x14 => "Virtualization exception",
                        0x1E => "Security exception",
                        _ => "Unknown Interrupt",
                    };

                    string = string + "\n" + &format!("{:<6X}{:<16}{}", interrupt, count, description);
                }
            }
        }

        Ok(box VecResource::new(Url::from_str("interrupt:"), string.into_bytes()))
    }
}
