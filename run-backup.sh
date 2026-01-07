#!/bin/bash
set -e

echo password | pbcopy

ISO="${1:-build/aarch64/server-cranelift.iso.ok.bak}"
SHARE="${2:-/tmp/9p-share}"

echo "Starting Redox backup $ISO with 9p share at: $SHARE"
echo "Access from Redox: /scheme/9p.hostshare/"
echo "Press Ctrl-A X to exit QEMU"
echo

# Use -cpu cortex-a72 (emulated); -accel hvf -cpu host causes crashes
qemu-system-aarch64 -M virt -cpu cortex-a72 -m 2G \
    -bios tools/firmware/edk2-aarch64-code.fd \
    -drive file="$ISO",format=raw,id=hd0,if=none \
    -device virtio-blk-pci,drive=hd0 \
    -device virtio-9p-pci,fsdev=host0,mount_tag=hostshare \
    -fsdev local,id=host0,path="$SHARE",security_model=none \
    -device qemu-xhci -device usb-kbd \
    -nographic
