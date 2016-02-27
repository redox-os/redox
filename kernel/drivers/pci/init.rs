use disk::ahci::Ahci;
use disk::ide::Ide;

use env::Environment;

use schemes::file::FileScheme;

use super::config::PciConfig;
use super::common::class::*;
use super::common::subclass::*;
use super::common::vendorid::*;
use super::common::deviceid::*;
use super::common::programming_interface::*;

use audio::ac97::Ac97;
use audio::intelhda::IntelHda;

use network::rtl8139::Rtl8139;
use network::intel8254x::Intel8254x;

use usb::uhci::Uhci;
use usb::ohci::Ohci;
use usb::ehci::Ehci;
use usb::xhci::Xhci;

/// PCI device
pub unsafe fn pci_device(env: &mut Environment,
                         pci: PciConfig,
                         class_id: u8,
                         subclass_id: u8,
                         interface_id: u8,
                         vendor_code: u16,
                         device_code: u16) {
    match (class_id, subclass_id, interface_id) {
        (MASS_STORAGE, IDE, _) => {
            if let Some(module) = FileScheme::new(Ide::disks(pci)) {
                env.schemes.lock().push(module);
            }
        }
        (MASS_STORAGE, SATA, AHCI) => {
            if let Some(module) = FileScheme::new(Ahci::disks(pci)) {
                env.schemes.lock().push(module);
            }
        }
        (SERIAL_BUS, USB, UHCI) => env.schemes.lock().push(Uhci::new(pci)),
        (SERIAL_BUS, USB, OHCI) => env.schemes.lock().push(Ohci::new(pci)),
        (SERIAL_BUS, USB, EHCI) => env.schemes.lock().push(Ehci::new(pci)),
        (SERIAL_BUS, USB, XHCI) => env.schemes.lock().push(Xhci::new(pci)),
        _ => match (vendor_code, device_code) {
            (REALTEK, RTL8139) => env.schemes.lock().push(Rtl8139::new(pci)),
            (INTEL, GBE_82540EM) => env.schemes.lock().push(Intel8254x::new(pci)),
            (INTEL, AC97_82801AA) => env.schemes.lock().push(Ac97::new(pci)),
            (INTEL, AC97_ICH4) => env.schemes.lock().push(Ac97::new(pci)),
            (INTEL, INTELHDA_ICH6) => env.schemes.lock().push(IntelHda::new(pci)),
            _ => (),
        }
    }
}

/// Initialize PCI session
pub unsafe fn pci_init(env: &mut Environment) {
    for bus in 0..256 {
        for slot in 0..32 {
            for func in 0..8 {
                let mut pci = PciConfig::new(bus as u8, slot as u8, func as u8);
                let id = pci.read(0);

                if (id & 0xFFFF) != 0xFFFF {
                    let class_id = pci.read(8);

                    debug!(" * PCI {}, {}, {}: ID {:X} CL {:X}",
                           bus,
                           slot,
                           func,
                           id,
                           class_id);

                    for i in 0..6 {
                        let bar = pci.read(i * 4 + 0x10);
                        if bar > 0 {
                            debug!(" BAR{}: {:X}", i, bar);

                            pci.write(i * 4 + 0x10, 0xFFFFFFFF);
                            let size = (0xFFFFFFFF - (pci.read(i * 4 + 0x10) & 0xFFFFFFF0)) + 1;
                            pci.write(i * 4 + 0x10, bar);

                            if size > 0 {
                                debug!(" {}", size);
                            }
                        }
                    }

                    debugln!("");

                    pci_device(env,
                               pci,
                               ((class_id >> 24) & 0xFF) as u8,
                               ((class_id >> 16) & 0xFF) as u8,
                               ((class_id >> 8) & 0xFF) as u8,
                               (id & 0xFFFF) as u16,
                               ((id >> 16) & 0xFFFF) as u16);
                }
            }
        }
    }
}
