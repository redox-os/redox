use x86::io;

use device::serial::{COM1, COM2};
use time;

extern {
    fn irq_trigger(irq: u8);
}

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
    irq_trigger(0);

    {
        const PIT_RATE: u64 = 46500044;

        let mut offset = time::OFFSET.lock();
        let sum = offset.1 + PIT_RATE;
        offset.1 = sum % 1000000000;
        offset.0 += sum / 1000000000;
    }

    master_ack();
});

interrupt!(keyboard, {
    irq_trigger(1);
});

interrupt!(cascade, {
    irq_trigger(2);
    master_ack();
});

interrupt!(com2, {
    irq_trigger(3);
    COM2.lock().on_receive();
    master_ack();
});

interrupt!(com1, {
    irq_trigger(4);
    COM1.lock().on_receive();
    master_ack();
});

interrupt!(lpt2, {
    irq_trigger(5);
    master_ack();
});

interrupt!(floppy, {
    irq_trigger(6);
    master_ack();
});

interrupt!(lpt1, {
    irq_trigger(7);
    master_ack();
});

interrupt!(rtc, {
    irq_trigger(8);
    slave_ack();
});

interrupt!(pci1, {
    irq_trigger(9);
    slave_ack();
});

interrupt!(pci2, {
    irq_trigger(10);
});

interrupt!(pci3, {
    irq_trigger(11);
});

interrupt!(mouse, {
    irq_trigger(12);
});

interrupt!(fpu, {
    irq_trigger(13);
    slave_ack();
});

interrupt!(ata1, {
    irq_trigger(14);
    slave_ack();
});

interrupt!(ata2, {
    irq_trigger(15);
    slave_ack();
});
