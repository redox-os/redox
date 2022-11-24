# Redox 0.8.0 Hardware Compatibility

Updated on November 11, 2022 in preparation for the 0.8.0 release. Systems are
rated on a scale of 🚫 Broken, ⚠️ Booting, and ✅ Recommended. Broken means the
system cannot boot to a desktop, booting means the system boots to a desktop
but has issues, and recommended means the system provides all implemented
features.

## General

Due to incomplete USB support, desktops are generally not recommended.

- USB support is incomplete
- Wireless networking is not supported

## x86_64

Test performed using https://static.redox-os.org/img/x86_64/redox_desktop_x86_64_2022-11-11_629_livedisk.iso

### Lenovo IdeaPad Y510P

Status: ✅ Recommended

- Booted using both BIOS and UEFI
- Boots to desktop

### System76 Galago Pro (galp5)

Status: ✅ Recommended

- Booted using UEFI
- Boots to desktop

### System76 Lemur Pro (lemp9)

Status: ✅ Recommended

- Booted using UEFI
- Boots to desktop

### Asus X554L

Status: ⚠️ Booting

- Booted using BIOS
- Boots to desktop
- No audio, HDA driver cannot find output pins

### Dell XPS 13 (9350)

Status: ⚠️ Booting

- Booted using both BIOS and UEFI
- Boots to desktop
- NVMe driver livelocks

### HP Dev One

Status: ⚠️ Booting

- Booted using UEFI
- Boots to desktop
- No touchpad support, requires I2C HID

### System76 Oryx Pro (oryp10)

Status: ⚠️ Booting

- Booted using UEFI
- Boots to desktop
- No touchpad support, though it should be working

### System76 Pangolin (pang12)

Status: ⚠️ Booting

- Booted using UEFI
- Boots to desktop
- No touchpad support, requires I2C HID

### Toshiba Satellite L500

Status: ⚠️ Booting

- Booted using BIOS
- Correct video mode not offered, this is a firmware issue
- Boots to desktop
- No ethernet driver

### Lenovo G570

Status: 🚫 Broken

- Booted using BIOS
- Correct video mode not offered, this is a firmware issue
- Bootloader panics in alloc_zeroed_page_aligned

## i686

Test performed using https://static.redox-os.org/img/i686/redox_desktop_i686_2022-11-11_629_livedisk.iso

### Asus Eee PC 900

Status: ⚠️ Booting

- Booted using BIOS
- Correct video mode not offered, this is a firmware issue
- Boots to desktop
- No ethernet driver

### Dell XPS 13 (9350)

Status: ⚠️ Booting

- Booted using BIOS
- Boots to desktop
- NVMe driver livelocks

### Lenovo IdeaPad Y510P

Status: 🚫 Broken

- Booted using BIOS
- Panics on phys_to_virt overflow, probably having invalid mappings for 32-bit

### Panasonic Toughbook CF-18

Status: 🚫 Broken

- Booted using BIOS
- Hangs after PIT initialization

### Toshiba Satellite L500

Status: 🚫 Broken

- Booted using BIOS
- Correct video mode not offered, this is a firmware issue
- Panics on phys_to_virt overflow, probably having invalid mappings for 32-bit
