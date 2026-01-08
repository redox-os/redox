#!/bin/bash
set -e

echo password | pbcopy

cd "$(dirname "$0")"
ROOT="$(pwd)"

QCOW2="${1:-$ROOT/build/aarch64/dev.qcow2.works}"
BASE_ISO="$ROOT/build/aarch64/pure-rust-works.iso"
SHARE="${2:-$ROOT/share/}"

echo "Using $BASE_ISO with snapshot via qcow2: $QCOW2"
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
