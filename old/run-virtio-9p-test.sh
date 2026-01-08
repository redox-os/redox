#!/bin/bash
# Test script for virtio-9p filesystem in QEMU

set -e

ROOT=/opt/other/redox
ARCH=${1:-aarch64}
HOST_DIR=${2:-/tmp/redox-9p-share}

# Create the shared directory if it doesn't exist
mkdir -p "$HOST_DIR"
echo "Sharing host directory: $HOST_DIR"

# Create some test files
echo "Hello from host!" > "$HOST_DIR/hello.txt"
mkdir -p "$HOST_DIR/subdir"
echo "Nested file" > "$HOST_DIR/subdir/nested.txt"

# Find the ISO
ISO=$(ls -t "$ROOT/build/$ARCH"/{desktop,server}/redox-live.iso 2>/dev/null | head -1)
if [[ ! -f "$ISO" ]]; then
    echo "Error: No ISO found in $ROOT/build/$ARCH/"
    exit 1
fi
echo "Using ISO: $ISO"

# Find virtio-9pd binary
VIRTIO9PD="$ROOT/recipes/core/base/source/target/aarch64-unknown-redox-clif/release/virtio-9pd"
if [[ -f "$VIRTIO9PD" ]]; then
    echo "virtio-9pd found: $(ls -lh "$VIRTIO9PD" | awk '{print $5}')"
else
    echo "Warning: virtio-9pd not built yet"
fi

# QEMU command based on architecture
if [[ "$ARCH" == "aarch64" ]]; then
    QEMU=qemu-system-aarch64
    FIRMWARE="$ROOT/tools/firmware/edk2-aarch64-code.fd"
    MACHINE="-M virt -cpu cortex-a72"
    DRIVE="-device virtio-blk-pci,drive=hd0"
else
    QEMU=qemu-system-x86_64
    FIRMWARE="$ROOT/tools/firmware/edk2-x86_64-code.fd"
    MACHINE="-machine q35"
    DRIVE="-device ahci,id=ahci -device ide-hd,drive=hd0,bus=ahci.0"
fi

echo ""
echo "=== QEMU Command ==="
echo "Starting QEMU with virtio-9p device..."
echo "Mount tag: hostshare"
echo ""
echo "In Redox, if virtio-9pd works, you should see the scheme at /scheme/9p.hostshare/"
echo ""

$QEMU $MACHINE -m 2G \
    -bios "$FIRMWARE" \
    -drive file="$ISO",format=raw,id=hd0,if=none \
    $DRIVE \
    -device virtio-net-pci,netdev=net0 \
    -netdev user,id=net0 \
    -device qemu-xhci -device usb-kbd -device usb-tablet \
    -fsdev local,id=fsdev0,path="$HOST_DIR",security_model=mapped-xattr \
    -device virtio-9p-pci,fsdev=fsdev0,mount_tag=hostshare \
    -serial stdio \
    -display none
