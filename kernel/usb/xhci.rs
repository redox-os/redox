use alloc::boxed::Box;

use arch::memory::*;

use drivers::pci::config::PciConfig;

use core::mem::size_of;

use common::debug;
//For old code vvv
use common::debug::*;

use schemes::KScheme;

#[repr(packed)]
struct Ste {
    pub ptr: u64,
    pub length: u64,
}

#[repr(packed)]
struct Trb {
    pub data: u64,
    pub status: u32,
    pub control: u32,
}

impl Trb {
    pub fn new() -> Self {
        Trb {
            data: 0,
            status: 0,
            control: 0,
        }
    }

    pub fn from_type(trb_type: u32) -> Self {
        Trb {
            data: 0,
            status: 0,
            control: (trb_type & 0x3F) << 10,
        }
    }
}

pub struct Xhci {
    pub pci: PciConfig,
    pub base: usize,
    pub irq: u8,
}

impl KScheme for Xhci {
    fn on_irq(&mut self, irq: u8) {
        if irq == self.irq {
            debug::d("XHCI handle\n");
        }
    }
}

impl Xhci {
    pub unsafe fn new(mut pci: PciConfig) -> Box<Xhci> {
        let mut module = box Xhci {
            pci: pci,
            base: pci.read(0x10) as usize & 0xFFFFFFF0,
            irq: pci.read(0x3C) as u8 & 0xF,
        };
        module.init();
        module
    }

    pub unsafe fn init(&mut self) {
        debug::d("XHCI on: ");
        debug::dh(self.base);
        debug::d(" IRQ: ");
        debug::dbh(self.irq);
        debug::dl();

        self.pci.flag(4, 4, true); // Bus mastering

        let cap_base = self.base;
        let op_base = cap_base + *(cap_base as *mut u8) as usize;
        let db_base = cap_base + *((cap_base + 0x14) as *mut u32) as usize;
        let rt_base = cap_base + *((cap_base + 0x18) as *mut u32) as usize;

        d("CAP_BASE: ");
        dh(cap_base);

        d(" OP_BASE: ");
        dh(op_base);

        d(" DB_BASE: ");
        dh(db_base);

        d(" RT_BASE: ");
        dh(rt_base);
        dl();

        //Set FLADJ Frame Length Adjustment (optional?)
        //Set I/O memory maps (optional?)

        //Wait until the Controller Not Ready flag in USBSTS is 0
        let usbsts = (op_base + 0x04) as *mut u32;
        while *usbsts & (1 << 11) == (1 << 11) {
            d("Controller Not Ready\n");
        }
        d("Controller Ready ");
        dh(*usbsts as usize);
        dl();

        //Set Run/Stop to 0
        let usbcmd = op_base as *mut u32;
        *usbcmd = *usbcmd & 0xFFFFFFFE;

        while *usbsts & 1 == 0 {
            d("Command Not Ready\n");
        }

        d("Command Ready ");
        dh(*usbcmd as usize);
        dl();

        //Program the Max Device Slots Enabled in the CONFIG register
        let hcsparams1 = (cap_base + 0x04) as *const u32;
        let max_slots = *hcsparams1 & 0xFF;
        let max_ports = (*hcsparams1 >> 24) & 0xFF;

        d("Max Slots ");
        dd(max_slots as usize);
        dl();

        d("Max Ports ");
        dd(max_ports as usize);
        dl();

        let config = (op_base + 0x38) as *mut u32;
        *config = max_slots;
        d("Slots Enabled ");
        dd(*config as usize);
        dl();

        //Program the Device Context Base Address Array Pointer with a pointer to the Device Context Base Address Array
        let device_context_base_address_array = alloc(max_slots as usize * size_of::<u64>()) as *mut u64;
        for slot in 0..max_slots as isize {
            *device_context_base_address_array.offset(slot) = alloc(2048) as u64;
        }

        let dcbaap = (op_base + 0x30) as *mut u64;
        *dcbaap = device_context_base_address_array as u64;

        d("Set Device Context Base Address Array ");
        dh(*dcbaap as usize);
        dl();

        //Define the Command Ring Dequeue Pointer by programming the Command ring Control register with a pointer to the first Trb
        let command_ring_length = 256;
        let mut command_ring_offset = 0;
        let command_ring = alloc(command_ring_length * size_of::<Trb>()) as *mut Trb;
        for i in 0..command_ring_length {
            *command_ring.offset(i as isize) = Trb::new();
            d("."); //Timing issue?
        }
        dl();

        let crcr = (op_base + 0x18) as *mut u64;
        *crcr = command_ring as u64;
        d("Set Command Ring Dequeue Pointer ");
        dh(*crcr as usize);
        dl();

        //Define the Event Ring for interrupter 0
        let event_ring_segments = 1;
        let event_ring_segment_table = alloc(event_ring_segments * size_of::<Ste>()) as *mut Ste;
        let mut event_ring_dequeue = 0;

        for segment in 0..event_ring_segments {
            let ste = &mut *event_ring_segment_table.offset(segment as isize);
            ste.length = 256;
            ste.ptr = alloc(ste.length as usize * size_of::<Trb>()) as u64;

            for i in 0..ste.length as isize {
                *(ste.ptr as *mut Trb).offset(i) = Trb::new();
                dd(i as usize);
                d(" ");
            }
            dl();

            if segment == 0 {
                event_ring_dequeue = ste.ptr;
            }
        }

        let erstsz = (rt_base + 0x28) as *mut u32;
        *erstsz = event_ring_segments as u32;

        let erdp = (rt_base + 0x38) as *mut u64;
        *erdp = event_ring_dequeue;

        let erstba = (rt_base + 0x30) as *mut u64;
        *erstba = event_ring_segment_table as u64;

        d("Set Event Ring Segment Table ");
        dh(*erstba as usize);
        d(" ");
        dd(*erstsz as usize);
        d(" ");
        dh(*erdp as usize);
        dl();

        //Write the USBCMD to turn on the host controller by setting Run/Stop to 1
        let usbcmd = op_base as *mut u32;
        *usbcmd = *usbcmd | 1;

        while *usbsts & 1 != 0 {
            d("Not Running\n");
        }

        d("Running ");
        dh(*usbcmd as usize);
        dl();

        for i in 0..max_ports as usize {
            let portsc = (op_base + 0x400 + (0x10 * i)) as *mut u32;
            d("Port ");
            dd(i + 1);
            d(" is ");
            dh(*portsc as usize);
            dl();

            if *portsc & 1 == 1 {
                d("Connected\n");
            }
            if *portsc & 2 == 2 {
                d("Enabled\n");
            }
            if *portsc & 3 == 3 {
                d("Enabling slot\n");

                *command_ring.offset(command_ring_offset as isize) = Trb::from_type(23);
                command_ring_offset += 1;
                if command_ring_offset >= command_ring_length {
                    command_ring_offset = 0;
                }

                *command_ring.offset(command_ring_offset as isize) = Trb::from_type(9);
                command_ring_offset += 1;
                if command_ring_offset >= command_ring_length {
                   command_ring_offset = 0;
                }

                d("Write Doorbell\n");
                let doorbell = db_base as *mut u32;
                *doorbell = 0;
            }
        }
    }
}
