#!/bin/bash
# Run Redox with writable qcow2 disk image (persistent changes)
set -e
echo password | pbcopy
QCOW2="${1:-build/aarch64/server-cranelift.qcow2}"
SHARE="${2:-/tmp/9p-share}"

mkdir -p "$SHARE"
[ -f "$SHARE/test.txt" ] || echo "Hello from host filesystem via virtio-9p!" > "$SHARE/test.txt"

echo "Starting Redox with WRITABLE qcow2 disk"
echo "Disk image: $QCOW2"
echo "9p share: $SHARE"
echo "Changes to the disk will PERSIST across reboots"
echo "Press Ctrl-A X to exit QEMU"
echo

qemu-system-aarch64 -M virt -cpu cortex-a72 -m 2G \
    -bios tools/firmware/edk2-aarch64-code.fd \
    -drive file="$QCOW2",format=qcow2,id=hd0,if=none \
    -device virtio-blk-pci,drive=hd0 \
    -device virtio-9p-pci,fsdev=host0,mount_tag=hostshare \
    -fsdev local,id=host0,path="$SHARE",security_model=none \
    -device qemu-xhci -device usb-kbd \
    -nographic
