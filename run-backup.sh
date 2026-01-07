#!/bin/bash
# Run Redox with virtio-9p host filesystem sharing
set -e

echo password | pbcopy

ISO="${1:-build/aarch64/server-cranelift.iso.bak}"
SHARE="${2:-/tmp/9p-share}"

mkdir -p "$SHARE"

# Create test file if not exists
[ -f "$SHARE/test.txt" ] || echo "Hello from host filesystem via virtio-9p!" > "$SHARE/test.txt"

echo "Starting Redox with 9p share at: $SHARE"
echo "Access from Redox: /scheme/9p.hostshare/"
echo "Press Ctrl-A X to exit QEMU"
echo

# QEMU optimization flags for faster boot
ACCEL_OPTS=""
if [ "$(uname)" = "Darwin" ]; then
    ACCEL_OPTS="-accel hvf -cpu host -smp 4"
else
    ACCEL_OPTS="-accel kvm -cpu host -smp 4" 2>/dev/null || ACCEL_OPTS="-smp 4"
fi

# Note: no ramfb device - it causes resolution menu that blocks serial boot
qemu-system-aarch64 -M virt $ACCEL_OPTS -m 2G \
    -bios tools/firmware/edk2-aarch64-code.fd \
    -drive file="$ISO",format=raw,id=hd0,if=none \
    -device virtio-blk-pci,drive=hd0 \
    -device virtio-9p-pci,fsdev=host0,mount_tag=hostshare \
    -fsdev local,id=host0,path="$SHARE",security_model=none \
    -device qemu-xhci -device usb-kbd \
    -nographic
