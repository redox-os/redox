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
    if let Some(ref mut keyboard) = *PS2_KEYBOARD.lock(){
        keyboard.on_irq();
    }
    master_ack();
});

interrupt!(cascade, {
    print!("CASCADE\n");
    master_ack();
});

interrupt!(com2, {
    COM2.lock().on_receive();
    master_ack();
});

interrupt!(com1, {
    COM1.lock().on_receive();
    master_ack();
});

interrupt!(lpt2, {
    print!("LPT2\n");
    master_ack();
});

interrupt!(floppy, {
    print!("FLOPPY\n");
    master_ack();
});

interrupt!(lpt1, {
    print!("LPT1\n");
    master_ack();
});

interrupt!(rtc, {
    print!("RTC\n");
    slave_ack();
});

interrupt!(pci1, {
    print!("PCI1\n");
    slave_ack();
});

interrupt!(pci2, {
    print!("PCI2\n");
    slave_ack();
});

interrupt!(pci3, {
    print!("PCI3\n");
    slave_ack();
});

interrupt!(mouse, {
    if let Some(ref mut mouse) = *PS2_MOUSE.lock() {
        mouse.on_irq();
    }
    slave_ack();
});

interrupt!(fpu, {
    print!("FPU\n");
    slave_ack();
});

interrupt!(ata1, {
    print!("ATA1\n");
    slave_ack();
});

interrupt!(ata2, {
    print!("ATA2\n");
    slave_ack();
});
