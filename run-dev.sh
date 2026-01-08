#!/bin/bash
set -e

cd "$(dirname "$0")"
ROOT="$(pwd)"

QCOW2="${QCOW2:-$ROOT/build/aarch64/dev.qcow2}"
BASE_ISO="$ROOT/build/aarch64/pure-rust.iso"
SHARE="${SHARE:-$ROOT/share/}"
SOCK="${SOCK:-/tmp/redox-dev.sock}"
MONSOCK="${MONSOCK:-/tmp/redox-dev-mon.sock}"

# Create qcow2 from base ISO if it doesn't exist
if [[ ! -f "$QCOW2" ]]; then
    echo "Creating dev.qcow2 from $BASE_ISO..." >&2
    mkdir -p "$(dirname "$QCOW2")"
    qemu-img create -f qcow2 -b "$BASE_ISO" -F raw "$QCOW2" 4G
    echo "Created $QCOW2" >&2
fi

CPU="-accel tcg,thread=multi -cpu cortex-a72 -smp 4"

# Socket mode: for scripted/heredoc usage
if [[ "$1" == "-s" || "$1" == "--socket" ]]; then
    rm -f "$SOCK" "$MONSOCK"
    echo "Socket mode: $SOCK" >&2
    echo "Monitor: $MONSOCK" >&2
    echo "Connect: socat - unix-connect:$SOCK" >&2
    qemu-system-aarch64 -M virt $CPU -m 2G \
        -bios tools/firmware/edk2-aarch64-code.fd \
        -drive file="$QCOW2",format=qcow2,id=hd0,if=none \
        -device virtio-blk-pci,drive=hd0 \
        -device virtio-9p-pci,fsdev=host0,mount_tag=hostshare \
        -fsdev local,id=host0,path="$SHARE",security_model=none \
        -device qemu-xhci -device usb-kbd \
        -serial unix:"$SOCK",server,nowait \
        -monitor unix:"$MONSOCK",server,nowait \
        -nographic -display none &
    QEMU_PID=$!
    sleep 1
    echo "QEMU PID: $QEMU_PID" >&2
    echo "$SOCK"  # Output socket path for scripts
else
    # Interactive mode (default)
    echo "Using: $QCOW2" >&2
    echo "Snapshots: ./snapshot.sh [save|load|list]" >&2
    echo "Socket mode: $0 -s" >&2
    qemu-system-aarch64 -M virt $CPU -m 2G \
        -bios tools/firmware/edk2-aarch64-code.fd \
        -drive file="$QCOW2",format=qcow2,id=hd0,if=none \
        -device virtio-blk-pci,drive=hd0 \
        -device virtio-9p-pci,fsdev=host0,mount_tag=hostshare \
        -fsdev local,id=host0,path="$SHARE",security_model=none \
        -device qemu-xhci -device usb-kbd \
        -nographic
fi
