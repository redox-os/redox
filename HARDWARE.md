# Hardware Compatibility

This document tracks the current hardware compatibility of Redox.

- [Status](#status)
- [General](#general)
- [Template](#template)
- [x86-64](#x86-64)
    - [System76](#system76)
    - [Dell](#dell)
    - [HP](#hp)
    - [ASUS](#asus)
    - [Lenovo](#lenovo)
    - [Toshiba](#toshiba)
    - [Custom](#custom)
- [i686](#i686)
    - [Dell](#dell-1)
    - [ASUS](#asus-1)
    - [Lenovo](#lenovo-1)
    - [Toshiba](#toshiba-1)
    - [Panasonic](#panasonic)
    - [Custom](#custom-1)
- [ARM64](#arm64)
    - [Custom](#custom-2)

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

## Template

You will use this template to insert your computer.

- **Computer model**

```
Status - (Broken, Booting or Recommended)
Redox version - 0.0.0
Variant - (server-minimal, desktop-minimal, server, desktop, demo, dev or customized)
Image date - day-month-year

- Additional details goes here as items
```

## x86-64

Computers using a 64 bits Intel/AMD CPU.

### System76

- **System76 Galago Pro (galp5)**

```
Status - Recommended
Redox version - 0.8.0
Variant - desktop
Image date - 11-11-2022

- Booted using UEFI
- Boots to desktop
```

- **System76 Lemur Pro (lemp9)**

```
Status - Recommended
Redox version - 0.8.0
Variant - desktop
Image date - 11-11-2022

- Booted using UEFI
- Boots to desktop
```

- **System76 Oryx Pro (oryp10)**

```
Status - Booting
Redox version - 0.8.0
Variant - desktop
Image date - 11-11-2022

- Booted using UEFI
- Boots to desktop
- No touchpad support, though it should be working
```

- **System76 Pangolin (pang12)**

```
Status - Booting
Redox version - 0.8.0
Variant - desktop
Image date - 11-11-2022

- Booted using UEFI
- Boots to desktop
- No touchpad support, requires I2C HID
```

### Dell

- **Dell XPS 13 (9350)**

```
Status - Booting
Redox version - 0.8.0
Variant - desktop
Image date - 11-11-2022

- Booted using both BIOS and UEFI
- Boots to desktop
- NVMe driver livelocks
```

### HP

- **HP Dev One**

```
Status - Booting
Redox version - 0.8.0
Variant - desktop
Image date - 11-11-2022

- Booted using UEFI
- Boots to desktop
- No touchpad support, requires I2C HID
```

- **HP EliteBook 2570p**

```
Status - Broken
Redox version - 0.8.0
Variant - demo
Image date - 2022-11-23

- UEFI Hybrid & Native boot don't work
- Booted using 'legacy'
- Gets to resolution selection  
- Fails assert in `src/os/bios/mod.rs:77` after selecting resolution
```

### BEELINK

- **BEELINK U59**

```
Status - Broken
Redox version - 0.8.x 
Variant - server
Image date - 30-05-2024

- Aborts after panic in xhcid
```

### ASUS

- **ASUS PN41**

```
Status - Broken
Redox version - 0.8.x 
Variant - server
Image date - 30-05-2024

- Aborts after panic in xhcid
```

- **ASUS X554L**

```
Status - Booting
Redox version - 0.8.0
Variant - desktop
Image date - 11-11-2022

- Booted using BIOS
- Boots to desktop
- No audio, HDA driver cannot find output pins
```

- **ASUS ROG g55vw**

```
Satus - Booting
Redox version - 0.8.0
Variant - desktop
Image date - 11-11-2023

- Booted using BIOS
- Boots to desktop
- UEFI panic in SETUP
```

### Lenovo

- **Lenovo IdeaPad Y510P**

```
Status - Recommended
Redox version - 0.8.0
Variant - desktop
Image date - 11-11-2022

- Booted using both BIOS and UEFI
- Boots to desktop
```

- **Lenovo G570**

```
Status - Broken
Redox version - 0.8.0
Variant - desktop
Image date - 11-11-2022

- Booted using BIOS
- Correct video mode not offered, this is a firmware issue
- Bootloader panics in alloc_zeroed_page_aligned
```

### Toshiba

- **Toshiba Satellite L500**

```
Status - Booting
Redox version - 0.8.0
Variant - desktop
Image date - 11-11-2022

- Booted using BIOS
- Correct video mode not offered, this is a firmware issue
- Boots to desktop
- No ethernet driver
```

### Custom

If you have a customized computer, put it here.



## i686

Computers with a 32 bits Intel/AMD CPU.

### Dell

- **Dell XPS 13 (9350)**

```
Status - Booting
Redox version - 0.8.0
Variant - desktop
Image date - 11-11-2022

- Booted using BIOS
- Boots to desktop
- NVMe driver livelocks
```

### ASUS

- **ASUS Eee PC 900**

```
Status - Booting
Redox version - 0.8.0
Variant - desktop
Image date - 11-11-2022

- Booted using BIOS
- Correct video mode not offered, this is a firmware issue
- Boots to desktop
- No ethernet driver
```

### Lenovo

- **Lenovo IdeaPad Y510P**

```
Status - Broken
Redox version - 0.8.0
Variant - desktop
Image date - 11-11-2022

- Booted using BIOS
- Panics on phys_to_virt overflow, probably having invalid mappings for 32-bit
```

### Toshiba

- **Toshiba Satellite L500**

```
Status - Broken
Redox version - 0.8.0
Variant - desktop
Image date - 11-11-2022

- Booted using BIOS
- Correct video mode not offered, this is a firmware issue
- Panics on phys_to_virt overflow, probably having invalid mappings for 32-bit
```

### Panasonic

- **Panasonic Toughbook CF-18**

```
Status - Broken
Redox version - 0.8.0
Variant - desktop
Image date - 11-11-2022

- Booted using BIOS
- Hangs after PIT initialization
```

### Custom

If you have a customized computer, put it here.



## ARM64

Computers using a 64 bits ARM CPU.

### Raspberry Pi

- **Raspberry Pi 3 Model B+**

```
Status - Booting
Redox version - 0.8.0
Variant - server
Image date - None

- Booted using Uboot
- Boots to UART serial console
- a bcm2835-sdhci/mmc driver
- pl011 UART
```

### Custom

If you have a customized ARM board, put it here.

