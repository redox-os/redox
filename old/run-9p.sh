#!/bin/bash
# Run Redox with virtio-9p host filesystem sharing
set -e

echo password | pbcopy

ISO="${1:-build/aarch64/server-cranelift.iso}"
SHARE="${2:-/tmp/9p-share}"

mkdir -p "$SHARE"

# Create test file if not exists
[ -f "$SHARE/test.txt" ] || echo "Hello from host filesystem via virtio-9p!" > "$SHARE/test.txt"

echo "Starting Redox with 9p share at: $SHARE"
echo "Access from Redox: /scheme/9p.hostshare/"
echo "Press Ctrl-A X to exit QEMU"
echo

# QEMU optimization flags for faster boot
CPU="-accel tcg,thread=multi -cpu cortex-a72" # WORKS!
# CPU="-accel hvf -cpu host" # BREAKS!!
# CPU="-cpu host" # why not alone?? needs accelerator!
# CPU="-accel hvf -cpu max"  # boots but breaks later on arrow keys? :
# kernel::context::memory:DEBUG -- Lacks grant
# kernel::arch::aarch64::interrupt::exception:ERROR -- FATAL: Not an SVC induced synchronous exception (ty=100100)


# Note: no ramfb device - it causes resolution menu that blocks serial boot
qemu-system-aarch64 -M virt $CPU -m 2G \
    -bios tools/firmware/edk2-aarch64-code.fd \
    -drive file="$ISO",format=raw,id=hd0,if=none \
    -device virtio-blk-pci,drive=hd0 \
    -device virtio-9p-pci,fsdev=host0,mount_tag=hostshare \
    -fsdev local,id=host0,path="$SHARE",security_model=none \
    -device qemu-xhci -device usb-kbd \
    -nographic
