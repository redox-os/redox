#!/bin/bash
set -e

cd "$(dirname "$0")"
ROOT="$(pwd)"

QCOW2="${1:-$ROOT/build/aarch64/dev.qcow2}"
BASE_ISO="$ROOT/build/aarch64/pure-rust.iso"
SHARE="${2:-$ROOT/share/}"

# Create qcow2 from base ISO if it doesn't exist
if [[ ! -f "$QCOW2" ]]; then
    echo "Creating dev.qcow2 from $BASE_ISO..."
    mkdir -p "$(dirname "$QCOW2")"
    qemu-img create -f qcow2 -b "$BASE_ISO" -F raw "$QCOW2" 4G
    echo "Created $QCOW2 (backed by $BASE_ISO)"
fi

echo "Using qcow2: $QCOW2"
echo "Snapshots: ./snapshot.sh [save|load|list] - manage VM state"
echo ""

CPU="-accel tcg,thread=multi -cpu cortex-a72 -smp 4"

qemu-system-aarch64 -M virt $CPU -m 2G \
    -bios tools/firmware/edk2-aarch64-code.fd \
    -drive file="$QCOW2",format=qcow2,id=hd0,if=none \
    -device virtio-blk-pci,drive=hd0 \
    -device virtio-9p-pci,fsdev=host0,mount_tag=hostshare \
    -fsdev local,id=host0,path="$SHARE",security_model=none \
    -device qemu-xhci -device usb-kbd \
    -nographic
