use x86::io;

use device::ps2::PS2;
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
    io::outb(0x43, 0);
    let low = io::inb(0x40);
    let high = io::inb(0x40);
    let count = (high as u16) << 8 | (low as u16);
    let missed = 5370 - count;
    master_ack();
});

interrupt!(keyboard, {
    PS2.lock().on_keyboard();
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
    PS2.lock().on_mouse();
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
