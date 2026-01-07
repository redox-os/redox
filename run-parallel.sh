#!/bin/bash
# Run Redox in parallel - interactive mode on different serial socket
set -e

ISO="${1:-build/aarch64/server-cranelift.iso}"
SHARE="${2:-/tmp/9p-share-$$}"
SOCK="/tmp/redox-serial-$$.sock"

mkdir -p "$SHARE"
[ -f "$SHARE/test.txt" ] || echo "Hello from parallel instance $$" > "$SHARE/test.txt"

echo "Starting Redox PARALLEL instance (PID: $$)"
echo "9p share: $SHARE"
echo "Serial socket: $SOCK"
echo "Press Ctrl-A X to exit QEMU"
echo

# Clean up socket on exit
trap "rm -f $SOCK" EXIT

qemu-system-aarch64 -M virt -cpu cortex-a72 -m 2G \
    -bios tools/firmware/edk2-aarch64-code.fd \
    -drive file="$ISO",format=raw,id=hd0,if=none,readonly=on \
    -device virtio-blk-pci,drive=hd0 \
    -device virtio-9p-pci,fsdev=host0,mount_tag=hostshare \
    -fsdev local,id=host0,path="$SHARE",security_model=none \
    -device qemu-xhci -device usb-kbd \
    -nographic
