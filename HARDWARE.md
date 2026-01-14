# Hardware Compatibility

This document tracks the current hardware compatibility of Redox OS.

- [Why are hardware reports needed?](#why-are-hardware-reports-needed)
- [What if my computer is customized?](#what-if-my-computer-is-customized)
- [Status](#status)
- [General](#general)
- [Contribute to this document](#contribute-to-this-document)
    - [Template](#template)
    - [Table row ordering](#table-row-ordering)
- [Recommended](#recommended)
- [Booting](#booting)
- [Broken](#broken)

## Why are hardware reports needed?

Each computer model has different hardware interfaces, firmware implementations, and devices, which can cause the following problems:

- Boot bugs
- Lack of device support
- Performance degradation

These reports helps us to fix the problems above, your report may help to fix many computers affected by the same bugs or missing drivers.

## What if my computer is customized?

If your desktop is customized (common) you should use the "Custom" word on the "Vendor" category and insert the motherboard and CPU vendor/model in the "Model" category.

A customized laptop should only be reported if you replaced the original CPU, report the CPU vendor and model in the "Model" category.

We also recommend to add your `pciutils` log on [this](https://gitlab.redox-os.org/redox-os/base/-/blob/main/drivers/COMMUNITY-HW.md) document to help us with probable porting.

## Status

- **Recommended:** The operating system boots with video, sound, PS/2 or USB input, Ethernet, terminal and Orbital working.
- **Booting:** The operating system boots with some issues or lacking hardware support (write the issues and what supported hardware is not working in the "Report" section).
- **Broken:** The boot loader don't work or can't bootstrap the operating system.

## General

This section contain limitations that apply to any status.

- ACPI support is incomplete (some things are hardcoded on the kernel)
- Wi-Fi and Bluetooth aren't supported yet
- AMD, NVIDIA, and ARM GPU drivers aren't available yet (only BIOS VESA and UEFI GOP)
- I2C devices aren't supported yet (PS/2 or USB devices should be used)
- USB support varies on each device model because some USB devices require specific drivers (use input devices with stanrdized controls for more compatibility)
- Automatic operating system discovery is not implemented in the boot loader yet (remember this before installing Redox)

## Contribute to this document

To contribute to this document, learn how to create your GitLab account, follow the project-wide contribution guidelines and suggestions, please refer to the [CONTRIBUTING.md](./CONTRIBUTING.md) document.

### Template

You will use this template to insert your computer on the table.

```
|  |  |  |  |  |  |  |  |
```

The Redox image date should use the [ISO format](https://en.wikipedia.org/wiki/ISO_8601)

### Table row ordering

New reports should use an independent alphabetical order in the "Vendor" and "Model" table rows, for example:

```
| ASUS | ROG g55vw |
| ASUS | X554L |
| System76 | Galago Pro (galp5) |
| System76 | Lemur Pro (lemp9) |
```

A comes before S, R comes before X, G comes before L

Each "Vendor" has its own alphabetical order in "Model", independent from models from other vendor.

## Recommended

| **Vendor** | **Model** | **Redox Version** | **Image Date** | **Variant** | **CPU Architecture** | **Motherboard Firmware** | **Report** |
|------------|-----------|-------------------|----------------|-------------|----------------------|--------------------------|------------|
| Lenovo | IdeaPad Y510P | 0.8.0 | 2022-11-11 | desktop | x86-64 | BIOS, UEFI | Boots to Orbital |
| System76 | Galago Pro (galp5) | 0.8.0 | 2022-11-11 | desktop | x86-64 | UEFI | Boots to Orbital |
| System76 | Lemur Pro (lemp9) | 0.8.0 | 2022-11-11 | desktop | x86-64 | UEFI | Boots to Orbital |

## Booting

| **Vendor** | **Model** | **Redox Version** | **Image Date** | **Variant** | **CPU Architecture** | **Motherboard Firmware** | **Report** |
|------------|-----------|-------------------|----------------|-------------|----------------------|--------------------------|------------|
| ASUS | Eee PC 900 | 0.8.0 | 2022-11-11 | desktop | i686 | BIOS | Boots to Orbital, No ethernet driver, Correct video mode not offered (firmware issue) |
| ASUS | PRIME B350M-E (custom) | 0.9.0 | 2024-09-20 | desktop | x86-64 | UEFI | Partial support for the PS/2 keyboard, PS/2 mouse is broken |
| ASUS | ROG g55vw | 0.8.0 | 2023-11-11 | desktop | x86-64 | BIOS | Boots to Orbital, UEFI panic in SETUP |
| ASUS | X554L | 0.8.0 | 2022-11-11 | desktop | x86-64 | BIOS | Boots to Orbital, No audio, HDA driver cannot find output pins |
| ASUS | Vivobook 15 OLED (M1503Q) | 0.9.0 | 2025-08-04 | desktop | x86-64 | UEFI | Boots to Orbital, touchpad and usb do not work, cannot connect to the internet, right maximum display resolution 2880x1620 |
| Dell | XPS 13 (9350) | 0.8.0 | 2022-11-11 | desktop | i686 | BIOS | Boots to Orbital, NVMe driver livelocks |
| Dell | XPS 13 (9350) | 0.8.0 | 2022-11-11 | desktop | x86-64 | BIOS, UEFI | Boots to Orbital, NVMe driver livelocks |
| HP | Dev One | 0.8.0 | 2022-11-11 | desktop | x86-64 | UEFI | Boots to Orbital, No touchpad support, requires I2C HID |
| HP | EliteBook Folio 9480M | 0.9.0 | 2025-11-04 | desktop | x86-64 | UEFI | Boots to Orbital, touchpad and usb work, cannot connect to the Internet, install failed, right maximum display resolution 1600x900
| Lenovo | ThinkPad Yoga 260 Laptop - Type 20FE | 0.9.0 | 2024-09-07 | demo | x86-64 | UEFI | Boots to Orbital, No audio |
| Lenovo | Yoga S730-13IWL | 0.9.0 | 2024-11-09 | desktop | x86-64 | UEFI | Boots to Orbital, No trackpad or USB mouse input support |
| Raspberry Pi | 3 Model B+ | 0.8.0 | Unknown | server | ARM64 | U-Boot | Boots to UART serial console (pl011) |
| Samsung | Series 3 (NP350V5C) | 0.9.0 | 2025-08-04 | desktop | x86-64 | UEFI | Boots to Orbital, touchpad works, USB does not work, can connect to the Internet through LAN. Wrong maximum display resolution 1024x768 |
| System76 | Oryx Pro (oryp10) | 0.8.0 | 2022-11-11 | desktop | x86-64 | UEFI | Boots to Orbital, No touchpad support, though it should be working |
| System76 | Pangolin (pang12) | 0.8.0 | 2022-11-11 | desktop | x86-64 | UEFI | Boots to Orbital, No touchpad support, requires I2C HID |
| Toshiba | Satellite L500 | 0.8.0 | 2022-11-11 | desktop | x86-64 | BIOS | Boots to Orbital, No Ethernet driver, Correct video mode not offered (firmware issue) |

## Broken

| **Vendor** | **Model** | **Redox Version** | **Image Date** | **Variant** | **CPU Architecture** | **Motherboard Firmware** | **Report** |
|------------|-----------|-------------------|----------------|-------------|----------------------|--------------------------|------------|
| ASUS | PN41 | 0.8.0 | 2024-05-30 | server | x86-64 | Unknown | Aborts after panic in xhcid |
| BEELINK | U59 | 0.8.0 | 2024-05-30 | server | x86-64 | Unknown | Aborts after panic in xhcid |
| Framework | Laptop 16 (AMD Ryzen 7040 Series) | 0.9.0 | 2024-09-07 | server, demo | x86-64 | UEFI | Black screen and unresponsive after the bootloader and resolution selection |
| HP | Compaq nc6120 | 0.9.0 | 2024-11-08 | desktop, server | i686 | BIOS | Unloads into memory at a rate slower than 1MB/s after selecting resolution. When unloading is complete the logger initializes and crashes after kernel::acpi, some information about APIC is printed. Boot logs do not progress after this point. |
| HP | EliteBook 2570p | 0.8.0 | 2022-11-23 | demo | x86-64 | BIOS (CSM mode?) | Gets to resolution selection, Fails assert in `src/os/bios/mod.rs:77` after selecting resolution |
| Lenovo | G570 | 0.8.0 | 2022-11-11 | desktop | x86-64 | BIOS | Bootloader panics in `alloc_zeroed_page_aligned`, Correct video mode not offered (firmware issue) |
| Lenovo | IdeaPad Y510P | 0.8.0 | 2022-11-11 | desktop | i686 | BIOS | Panics on `phys_to_virt overflow`, probably having invalid mappings for 32-bit |
| Lenovo | ThinkCentre M83 | 0.9.0 | 2025-11-09 | desktop | x86_64 | UEFI | Presents user with a set of display resolution options. After user selects an option, it takes a long time for the "live" thing to load all the way to 647MiB. Once it does reach 647MiB, however, it dumps a bunch of logs onto the screen. Those logs also happen to be offset so that the leftmost portion of all text "exists" past the leftmost part of the screen, resulting in the logs being only partially visible. The logs appear to include (among other things) 1. "thread 'main' (1) panicked at acpid/src/acpi.rs:256:68: Called `Result::unwrap()` on an `Err` value: Aml(NoCurrentOp)"; 2. "thread 'main' (1) panicked at acpid/src/main.rs:147:39:acpid: failed to daemonize: Error `I/O error` 5"; 3. "... [@hwd:40 ERROR] failed to probe with error No such device (os error 19)..."; etc. |
| Panasonic | Toughbook CF-18 | 0.8.0 | 2022-11-11 | desktop | i686 | BIOS | Hangs after PIT initialization |
| Toshiba | Satellite L500 | 0.8.0 | 2022-11-11 | desktop | i686 | BIOS | Correct video mode not offered (firmware issue), Panics on `phys_to_virt overflow`, probably having invalid mappings for 32-bit |
| XMG (Schenker) | Apex 17 (M21) | 0.9.0 | 2024-09-30 | demo, server | x86-64 | UEFI | After selecting resolution, (release) repeats `...::interrupt::irq::ERROR -- Local apic internal error: ESR=0x40` a few times before it freezes; (daily) really slowly prints statements from `...::rmm::INFO` before it abruptly aborts |

