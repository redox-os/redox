# Hardware Compatibility

This document tracks the current hardware compatibility of Redox.

- [Status](#status)
- [General](#general)
- [Hardware](#hardware)

## Status

- Broken - The system can't boot.
- Booting - The system boots with some issues.
- Recommended - The system start with all features working.

## General

This section cover things to consider.

- ACPI support is incomplete (some things are hardcoded on the kernel)
- Only USB input devices are supported
- Wi-Fi is not supported
- GPU drivers aren't supported (only VESA and UEFI GOP)
- Automatic operating system discovery on boot loader is not implemented (remember this before installing Redox)

## Hardware

|Instruction Set Architecture|Brand       |Model                            |Status     |Redox Version|Variant     |Image Date|Firmware Booting                                   |Boots To                                                                    |Report                                                                    |
|----------------------------|------------|---------------------------------|-----------|-------------|------------|----------|---------------------------------------------------|----------------------------------------------------------------------------|----------------------------------------------------------------------------|
|x86-64                      |System76    |Galago Pro (galp5)               |Recommended|0.8.0        |Desktop     |2022.11.11|UEFI                                               |Desktop                                                                     |                                                                            |
|x86-64                      |System76    |Lemur Pro (lemp9)                |Recommended|0.8.0        |Desktop     |2022.11.11|UEFI                                               |Desktop                                                                     |                                                                            |
|x86-64                      |Lenovo      |IdeaPad Y510P                    |Recommended|0.8.0        |Desktop     |2022.11.11|BIOS and UEFI                                      |Desktop                                                                     |                                                                            |
|x86-64                      |System76    |Oryx Pro (oryp10)                |Booting    |0.8.0        |Desktop     |2022.11.11|UEFI                                               |Desktop                                                                     |No touchpad support, thought it should be working                           |
|x86-64                      |System76    |Pangolin (pang12)                |Booting    |0.8.0        |Desktop     |2022.11.11|UEFI                                               |Desktop                                                                     |No touchpad support, requires I2C HID                                       |
|x86-64                      |Dell        |XPS 13 (9350)                    |Booting    |0.8.0        |Desktop     |2022.11.11|BIOS and UEFI                                      |Desktop                                                                     |NVMe driver livelocks                                                       |
|x86-64                      |HP          |Dev One                          |Booting    |0.8.0        |Desktop     |2022.11.11|UEFI                                               |Desktop                                                                     |No touchpad support, requires I2C HID                                       |
|x86-64                      |ASUS        |X554L                            |Booting    |0.8.0        |Desktop     |2022.11.11|BIOS                                               |Desktop                                                                     |No audio, HDA driver cannot find output pins                                |
|x86-64                      |ASUS        |ROG g55vw                        |Booting    |0.8.0        |Desktop     |2023.11.11|BIOS                                               |Desktop                                                                     |UEFI panic in SETUP                                                         |
|i686                        |Dell        |XPS 13 (9350)                    |Booting    |0.8.0        |Desktop     |2022.11.11|BIOS                                               |Desktop                                                                     |NVMe driver livelocks                                                       |
|x86-64                      |Toshiba     |Satellite L500                   |Booting    |0.8.0        |Desktop     |2022.11.11|BIOS                                               |Desktop (Correct video mode not offered, this is a firmware issue)          |No ethernet driver                                                          |
|i686                        |ASUS        |Eee PC 900                       |Booting    |0.8.0        |Desktop     |2022.11.11|BIOS                                               |Desktop (Correct video mode not offered, this is a firmware issue)          |No ethernet driver                                                          |
|arm64                       |Raspberry Pi|3 Model B+                       |Booting    |0.8.0        |Server      |None      |Uboot                                              |UART serial console                                                         |a bcm2835-sdhci/mmc driver, pl011 UART                                      |
|x86-64                      |HP          |EliteBook 2570p                  |Broken     |0.8.0        |Demo        |2022.11.23|Legacy Works (UEFI Hybrid & Native boot don't work)|Resolution Selection Screen                                                 |Fails assert in `src/os/bios/mod.rs:77` after selecting resolution          |
|x86-64                      |Beelink     |U59                              |Broken     |0.8.x        |Server      |2024.05.30|                                                   |                                                                            |Aborts after panic in xhcid                                                 |
|x86-64                      |ASUS        |PN41                             |Broken     |0.8.x        |Server      |2024.05.30|                                                   |                                                                            |Aborts after panic in xhcid                                                 |
|x86-64                      |Lenovo      |G570                             |Broken     |0.8.0        |Desktop     |2022.11.11|BIOS                                               |Correct video mode not offered, this is a firmware issue                    |Bootloader panics in alloc_zeroed_page_alligned                             |
|x86-64                      |Framework   |Laptop 16 (AMD Ryzen 7040 Series)|Broken     |0.9.0        |Demo, Server|2024.07.09|UEFI                                               |Blank screen and unresponsive after the bootloader and resolution selection |                                                                            |
|i686                        |Lenovo      |IdeaPad Y510P                    |Broken     |0.8.0        |Desktop     |2022.11.11|BIOS                                               |Panics on phys_to_virt overflow, probably having invalid mappings for 32-bit|                                                                            |
|i686                        |Toshiba     |Satellite L500                   |Broken     |0.8.0        |Desktop     |2022.11.11|BIOS                                               |Correct video mode not offered, this is a firmware issue                    |Panics on phys_to_virt overflow, probably having invalid mappings for 32-bit|
