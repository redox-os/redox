#!/usr/bin/env bash

#TODO: move to installer

function usage {
    echo "$0 [disk] [filesystem]" >&2
    exit 1
}

if [ -z "$1" ]
then
    echo "invalid disk '$1'" >&2
    usage
fi
DISK="$1"

if [ ! -f "$2" ]
then
    echo "invalid filesystem '$2'" >&2
    usage
fi
FILESYSTEM="$2"

MiB=1048576

set -ex

# Calculate filesystem partition size
fs_disk_size="$(du -m "${FILESYSTEM}" | cut -f1)"
fs_disk_blkcount="$(expr "${fs_disk_size}" \* "$(expr "${MiB}" / 512)")"
fs_start="2048"
fs_end="$(expr "${fs_start}" + "${fs_disk_blkcount}")"
fs_last="$(expr "${fs_end}" - 1)"

if [ -n "${BOOTLOADER_EFI}" ]
then
    # Calculate EFI partition size
    efi_disk_size="$(expr "$(du -m "${BOOTLOADER_EFI}" | cut -f1)" + 1)"
    efi_disk_blkcount="$(expr "$efi_disk_size" \* "$(expr "${MiB}" / 512)")"
    efi_start="${fs_end}"
    efi_end="$(expr "${efi_start}" + "${efi_disk_blkcount}")"
    efi_last="$(expr "${efi_end}" - 1)"

    # Populate an EFI system partition
    dd if=/dev/zero of="${DISK}.esp" bs="${MiB}" count="${efi_disk_size}"
    mkfs.vfat "${DISK}.esp"
    mmd -i "${DISK}.esp" efi
    mmd -i "${DISK}.esp" efi/boot
    mcopy -i "${DISK}.esp" "${BOOTLOADER_EFI}" "::${BOOTLOADER_EFI_PATH}"

    disk_size="$(expr "${fs_disk_size}" + "${efi_disk_size}" + 2)"
else
    disk_size="$(expr "${fs_disk_size}" + 2)"
fi

# Create the disk
dd if=/dev/zero of="${DISK}" bs="${MiB}" count="${disk_size}"

# Create partition table
"${PARTED}" -s -a minimal "${DISK}" mklabel gpt
"${PARTED}" -s -a minimal "${DISK}" mkpart redox ext4 "${fs_start}s" "${fs_last}"s
if [ -n "${BOOTLOADER_EFI}" ]
then
    "${PARTED}" -s -a minimal "${DISK}" mkpart EFI fat32 "${efi_start}"s "${efi_last}s"
    "${PARTED}" -s -a minimal "${DISK}" set 2 boot on
    "${PARTED}" -s -a minimal "${DISK}" set 2 esp on
fi

# Write the partitions
dd if="${FILESYSTEM}" of="${DISK}" bs=512 seek="${fs_start}" count="${fs_disk_blkcount}" conv=notrunc
if [ -n "${BOOTLOADER_EFI}" ]
then
    dd if="${DISK}.esp" of="${DISK}" bs=512 seek="${efi_start}" count="${efi_disk_blkcount}" conv=notrunc
fi

# Write BIOS bootloader if applicable
if [ -n "${BOOTLOADER_BIOS}" ]
then
	dd if="${BOOTLOADER_BIOS}" of="${DISK}" bs=1 count=446 conv=notrunc
	dd if="${BOOTLOADER_BIOS}" of="${DISK}" bs=512 skip=34 seek=34 conv=notrunc
fi
