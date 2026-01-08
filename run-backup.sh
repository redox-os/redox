#!/bin/bash
set -e

# EMERGENCY FALLBACK ONLY - DO NOT USE FOR DEVELOPMENT
# This script has a known-working configuration. DO NOT MODIFY.
# For development, use run-debug.sh instead.
# If you broke something, undo your changes immediately.

echo "=== BACKUP/EMERGENCY SCRIPT ==="
echo "Usage: ./run-backup.sh [ISO] [SHARE]"
echo "WARNING: This is for emergency fallback only, not for development!"
echo ""

echo password | pbcopy

ISO="${1:-build/aarch64/pure-rust.iso.ok.DONT_TOUCH.bak}"
SHARE="${2:-/tmp/9p-backup}"

echo "Starting Redox backup $ISO with 9p share at: $SHARE"
echo "Access from Redox: /scheme/9p.hostshare/"
echo "Press Ctrl-A X to exit QEMU"
echo

# -cpu max tells QEMU to emulate a CPU with all features the accelerator supports.
# CPU="-cpu cortex-a72"  # WORKS!
CPU="-accel tcg,thread=multi -cpu cortex-a72 -smp 4" # WORKS!
# CPU="-M virt,gic-version=2 -accel hvf -cpu host" # FATAL EXCEPTION grant
# CPU="-M virt,virtualization=off -accel hvf -cpu host" # BREAKS
# CPU="-M virt,highmem=off -accel hvf -cpu host" # sometimes crashes?
# CPU="-M virt,highmem=off -accel hvf -cpu host -smp 4"
# CPU="-accel hvf -cpu host" # BREAKS!!
# CPU="-cpu host" # why not alone? needs accelerator!
# CPU="-accel hvf -cpu max"  # boots but breaks later on arrow keys? :
# kernel::context::memory:DEBUG -- Lacks grant
# kernel::arch::aarch64::interrupt::exception:ERROR -- FATAL: Not an SVC induced synchronous exception (ty=100100)


# Use -cpu cortex-a72 (emulated); -accel hvf -cpu host causes crashes
qemu-system-aarch64 -M virt $CPU -m 2G \
    -bios tools/firmware/edk2-aarch64-code.fd \
    -drive file="$ISO",format=raw,id=hd0,if=none \
    -device virtio-blk-pci,drive=hd0 \
    -device virtio-9p-pci,fsdev=host0,mount_tag=hostshare \
    -fsdev local,id=host0,path="$SHARE",security_model=none \
    -device qemu-xhci -device usb-kbd \
    -nographic
