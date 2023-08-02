# Hardware Compatibility

This document tracks the current hardware compatibility of Redox.

- [Status](#status)
- [General](#general)
- [x86_64](#x86_64)
    - [System76](#system76)
    - [Dell](#dell)
    - [HP](#hp)
    - [ASUS](#asus)
    - [Lenovo](#lenovo)
    - [Toshiba](#toshiba)
- [i686](#i686)
    - [Dell](#dell-1)
    - [ASUS](#asus-1)
    - [Lenovo](#lenovo-1)
    - [Toshiba](#toshiba-1)
    - [Panasonic](#panasonic)

## Status

- Broken - The system can't boot.
- Booting - The system boots with some issues.
- Recommended - The system start with all features working.

## General

Due to incomplete USB support, desktops are generally not recommended.

- USB support is incomplete.
- Wireless networking is not supported.

## x86_64

Computers using a 64 bits Intel/AMD CPU.

Test performed using https://static.redox-os.org/img/x86_64/redox_desktop_x86_64_2022-11-11_629_livedisk.iso

### System76

- **System76 Galago Pro (galp5)**

- Status - Recommended
- Redox version - 0.8.0
- Variant - desktop
- Image date - 11-11-2022

- Booted using UEFI
- Boots to desktop

- **System76 Lemur Pro (lemp9)**

- Status - Recommended
- Redox version - 0.8.0
- Variant - desktop
- Image date - 11-11-2022

- Booted using UEFI
- Boots to desktop

- **System76 Oryx Pro (oryp10)**

- Status - Booting
- Redox version - 0.8.0
- Variant - desktop
- Image date - 11-11-2022

- Booted using UEFI
- Boots to desktop
- No touchpad support, though it should be working

- **System76 Pangolin (pang12)**

- Status - Booting
- Redox version - 0.8.0
- Variant - desktop
- Image date - 11-11-2022

- Booted using UEFI
- Boots to desktop
- No touchpad support, requires I2C HID

### Dell

- **Dell XPS 13 (9350)**

- Status - Booting
- Redox version - 0.8.0
- Variant - desktop
- Image date - 11-11-2022

- Booted using both BIOS and UEFI
- Boots to desktop
- NVMe driver livelocks

### HP

- **HP Dev One**

- Status - Booting
- Redox version - 0.8.0
- Variant - desktop
- Image date - 11-11-2022

- Booted using UEFI
- Boots to desktop
- No touchpad support, requires I2C HID

### ASUS

- **ASUS X554L**

- Status - Booting
- Redox version - 0.8.0
- Variant - desktop
- Image date - 11-11-2022

- Booted using BIOS
- Boots to desktop
- No audio, HDA driver cannot find output pins

### Lenovo

- **Lenovo IdeaPad Y510P**

- Status - Recommended
- Redox version - 0.8.0
- Variant - desktop
- Image date - 11-11-2022

- Booted using both BIOS and UEFI
- Boots to desktop

- **Lenovo G570**

- Status - Broken
- Redox version - 0.8.0
- Variant - desktop
- Image date - 11-11-2022

- Booted using BIOS
- Correct video mode not offered, this is a firmware issue
- Bootloader panics in alloc_zeroed_page_aligned

### Toshiba

- **Toshiba Satellite L500**

- Status - Booting
- Redox version - 0.8.0
- Variant - desktop
- Image date - 11-11-2022

- Booted using BIOS
- Correct video mode not offered, this is a firmware issue
- Boots to desktop
- No ethernet driver

## i686

Computers with a 32 bits Intel/AMD CPU.

Test performed using https://static.redox-os.org/img/i686/redox_desktop_i686_2022-11-11_629_livedisk.iso

### Dell

- **Dell XPS 13 (9350)**

- Status - Booting
- Redox version - 0.8.0
- Variant - desktop
- Image date - 11-11-2022

- Booted using BIOS
- Boots to desktop
- NVMe driver livelocks

### ASUS

- **ASUS Eee PC 900**

- Status - Booting
- Redox version - 0.8.0
- Variant - desktop
- Image date - 11-11-2022

- Booted using BIOS
- Correct video mode not offered, this is a firmware issue
- Boots to desktop
- No ethernet driver

### Lenovo

- **Lenovo IdeaPad Y510P**

- Status - Broken
- Redox version - 0.8.0
- Variant - desktop
- Image date - 11-11-2022

- Booted using BIOS
- Panics on phys_to_virt overflow, probably having invalid mappings for 32-bit

### Toshiba

- **Toshiba Satellite L500**

- Status - Broken
- Redox version - 0.8.0
- Variant - desktop
- Image date - 11-11-2022

- Booted using BIOS
- Correct video mode not offered, this is a firmware issue
- Panics on phys_to_virt overflow, probably having invalid mappings for 32-bit

### Panasonic

- **Panasonic Toughbook CF-18**

- Status - Broken
- Redox version - 0.8.0
- Variant - desktop
- Image date - 11-11-2022

- Booted using BIOS
- Hangs after PIT initialization
