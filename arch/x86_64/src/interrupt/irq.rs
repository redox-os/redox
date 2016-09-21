use spin::Mutex;
use x86::io;

use device::serial::{COM1, COM2};

pub static ACKS: Mutex<[usize; 16]> = Mutex::new([0; 16]);
pub static COUNTS: Mutex<[usize; 16]> = Mutex::new([0; 16]);

#[inline(always)]
unsafe fn master_ack() {
    io::outb(0x20, 0x20);
}

#[inline(always)]
unsafe fn slave_ack() {
    io::outb(0xA0, 0x20);
    master_ack();
}

pub unsafe fn acknowledge(irq: usize) {
    if irq >= 8 {
        slave_ack();
    } else {
        master_ack();
    }
}

interrupt!(pit, {
    COUNTS.lock()[0] += 1;
    master_ack();
});

interrupt!(keyboard, {
    COUNTS.lock()[1] += 1;
});

interrupt!(cascade, {
    COUNTS.lock()[2] += 1;
    master_ack();
});

interrupt!(com2, {
    COUNTS.lock()[3] += 1;
    COM2.lock().on_receive();
    master_ack();
});

interrupt!(com1, {
    COUNTS.lock()[4] += 1;
    COM1.lock().on_receive();
    master_ack();
});

interrupt!(lpt2, {
    COUNTS.lock()[5] += 1;
    master_ack();
});

interrupt!(floppy, {
    COUNTS.lock()[6] += 1;
    master_ack();
});

interrupt!(lpt1, {
    COUNTS.lock()[7] += 1;
    master_ack();
});

interrupt!(rtc, {
    COUNTS.lock()[8] += 1;
    slave_ack();
});

interrupt!(pci1, {
    COUNTS.lock()[9] += 1;
    slave_ack();
});

interrupt!(pci2, {
    COUNTS.lock()[10] += 1;
    slave_ack();
});

interrupt!(pci3, {
    COUNTS.lock()[11] += 1;
    slave_ack();
});

interrupt!(mouse, {
    COUNTS.lock()[12] += 1;
});

interrupt!(fpu, {
    COUNTS.lock()[13] += 1;
    slave_ack();
});

interrupt!(ata1, {
    COUNTS.lock()[14] += 1;
    slave_ack();
});

interrupt!(ata2, {
    COUNTS.lock()[15] += 1;
    slave_ack();
});
