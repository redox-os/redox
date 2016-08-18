use core::mem;

use interrupt::halt;

pub static mut IDTR: IdtDescriptor = IdtDescriptor {
    size: 0,
    offset: 0
};

pub static mut IDT: [IdtEntry; 256] = [IdtEntry::new(); 256];

pub unsafe fn init() {
    for entry in IDT[0..32].iter_mut() {
        entry.set_flags(IDT_PRESENT | IDT_RING_0 | IDT_INTERRUPT);
        entry.set_offset(8, exception as usize);
    }
    IDT[13].set_offset(8, protection_fault as usize);
    IDT[14].set_offset(8, page_fault as usize);
    for entry in IDT[32..].iter_mut() {
        entry.set_flags(IDT_PRESENT | IDT_RING_0 | IDT_INTERRUPT);
        entry.set_offset(8, blank as usize);
    }
    IDTR.set_slice(&IDT);

    init_ap();
}

pub unsafe fn init_ap() {
    IDTR.load();
}

interrupt!(blank, {

});

interrupt!(exception, {
    println!("EXCEPTION");
    loop {
        halt();
    }
});

interrupt_error!(protection_fault, {
    println!("PROTECTION FAULT");
    loop {
        halt();
    }
});

interrupt_error!(page_fault, {
    println!("PAGE FAULT");
    loop {
        halt();
    }
});

bitflags! {
    pub flags IdtFlags: u8 {
        const IDT_PRESENT = 1 << 7,
        const IDT_RING_0 = 0 << 5,
        const IDT_RING_1 = 1 << 5,
        const IDT_RING_2 = 2 << 5,
        const IDT_RING_3 = 3 << 5,
        const IDT_SS = 1 << 4,
        const IDT_INTERRUPT = 0xE,
        const IDT_TRAP = 0xF,
    }
}

#[repr(packed)]
pub struct IdtDescriptor {
    size: u16,
    offset: u64
}

impl IdtDescriptor {
    pub fn set_slice(&mut self, slice: &'static [IdtEntry]) {
        self.size = (slice.len() * mem::size_of::<IdtEntry>() - 1) as u16;
        self.offset = slice.as_ptr() as u64;
    }

    pub unsafe fn load(&self) {
        asm!("lidt [rax]" : : "{rax}"(self as *const _ as usize) : : "intel", "volatile");
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(packed)]
pub struct IdtEntry {
    offsetl: u16,
    selector: u16,
    zero: u8,
    attribute: u8,
    offsetm: u16,
    offseth: u32,
    zero2: u32
}

impl IdtEntry {
    pub const fn new() -> IdtEntry {
        IdtEntry {
            offsetl: 0,
            selector: 0,
            zero: 0,
            attribute: 0,
            offsetm: 0,
            offseth: 0,
            zero2: 0
        }
    }

    pub fn set_flags(&mut self, flags: IdtFlags) {
        self.attribute = flags.bits;
    }

    pub fn set_offset(&mut self, selector: u16, base: usize) {
        self.selector = selector;
        self.offsetl = base as u16;
        self.offsetm = (base >> 16) as u16;
        self.offseth = (base >> 32) as u32;
    }
}
