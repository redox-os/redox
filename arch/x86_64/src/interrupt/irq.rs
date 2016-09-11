use x86::io;

use device::ps2::{PS2_KEYBOARD, PS2_MOUSE};
use device::serial::{COM1, COM2};

#[inline(always)]
unsafe fn master_ack() {
    io::outb(0x20, 0x20);
}

#[inline(always)]
unsafe fn slave_ack() {
    io::outb(0xA0, 0x20);
    master_ack();
}

interrupt!(pit, {
    master_ack();
});

interrupt!(keyboard, {
    master_ack();
    if let Some(ref mut keyboard) = *PS2_KEYBOARD.lock(){
        keyboard.on_irq();
    }
});

interrupt!(cascade, {
    master_ack();
    print!("CASCADE\n");
});

interrupt!(com2, {
    master_ack();
    COM2.lock().on_receive();
});

interrupt!(com1, {
    master_ack();
    COM1.lock().on_receive();
});

interrupt!(lpt2, {
    master_ack();
    print!("LPT2\n");
});

interrupt!(floppy, {
    master_ack();
    print!("FLOPPY\n");
});

interrupt!(lpt1, {
    master_ack();
    print!("LPT1\n");
});

interrupt!(rtc, {
    slave_ack();
    print!("RTC\n");
});

interrupt!(pci1, {
    slave_ack();
    print!("PCI1\n");
});

interrupt!(pci2, {
    slave_ack();
    print!("PCI2\n");
});

interrupt!(pci3, {
    slave_ack();
    print!("PCI3\n");
});

interrupt!(mouse, {
    slave_ack();
    if let Some(ref mut mouse) = *PS2_MOUSE.lock() {
        mouse.on_irq();
    }
});

interrupt!(fpu, {
    slave_ack();
    print!("FPU\n");
});

interrupt!(ata1, {
    slave_ack();
    print!("ATA1\n");
});

interrupt!(ata2, {
    slave_ack();
    print!("ATA2\n");
});
