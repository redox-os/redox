use common::debug::*;
use common::pci::*;

use programs::session::*;

pub struct XHCI {
    pub bus: usize,
    pub slot: usize,
    pub func: usize,
    pub base: usize,
    pub memory_mapped: bool,
    pub irq: u8
}

impl SessionDevice for XHCI {
    fn handle(&mut self, irq: u8){
        if irq == self.irq {
            d("XHCI handle");
        }
    }
}

impl XHCI {
    pub unsafe fn init(&self){
        d("XHCI on: ");
        dh(self.base);
        if self.memory_mapped {
            d(" memory mapped");
        }else{
            d(" port mapped");
        }
        d(" IRQ: ");
        dbh(self.irq);
        dl();

        pci_write(self.bus, self.slot, self.func, 0x04, pci_read(self.bus, self.slot, self.func, 0x04) | (1 << 2)); // Bus master

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

        d("Enabling Slots ");
        dd((*hcsparams1 & 0xFF) as usize);
        dl();

        let config = (op_base + 0x38) as *mut u32;
        dh(*config as usize);
        *config = *hcsparams1 & 0xFF;
        d(" ");
        dh(*config as usize);
        dl();

        //Program the Device Context Base Address Array Pointer with a pointer to the Device Context Base Address Array
        //Device the Command Ring Dequeue Pointer by programming the Command ring Control register with a pointer to the first TRB
        //Initialize interrupts (optional)
            //Allocate and initalize the MSI-X Message Table, setting the message address and message data, and enable the vectors. At least table vector entry 0 should be initialized
            //Allocate and initialize the MSI-X Pending Bit Array
            //Point the Table Offset and PBA Offsets in the MSI-X Capability Structure to the MSI-X Message Control Table and Pending Bit Array
            //Initialize the Message Control register in the MSI-X Capability Structure
            //Initialize each active interrupter by:
                //TODO: Pull from page 72
        //Write the USBCMD to turn on the host controller by setting Run/Stop to 1
    }
}
