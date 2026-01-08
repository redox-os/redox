#!/bin/bash
# Run Redox for parallel debugging - uses different ports to allow multiple instances
set -e

cd "$(dirname "$0")"
ROOT="$(pwd)"

INSTANCE="${1:-1}"
QCOW2="${QCOW2:-$ROOT/build/aarch64/dev.qcow2}"
BASE_ISO="$ROOT/build/aarch64/pure-rust.iso"
SHARE="${SHARE:-/tmp/9p-debug-$INSTANCE}"

# Create qcow2 from base ISO if it doesn't exist
if [[ ! -f "$QCOW2" ]]; then
    echo "Creating dev.qcow2 from $BASE_ISO..."
    mkdir -p "$(dirname "$QCOW2")"
    qemu-img create -f qcow2 -b "$BASE_ISO" -F raw "$QCOW2" 4G
fi

# Calculate unique ports based on instance
SERIAL_PORT=$((4440 + INSTANCE))
MONITOR_PORT=$((4450 + INSTANCE))
GDB_PORT=$((1234 + INSTANCE))

mkdir -p "$SHARE"
echo password | pbcopy

echo "Redox DEBUG #$INSTANCE"
echo "  qcow2: $QCOW2"
echo "  9p: $SHARE"
echo "  serial: telnet localhost $SERIAL_PORT"
echo "  monitor: telnet localhost $MONITOR_PORT"
echo "  gdb: localhost:$GDB_PORT (paused, use 'c' to continue)"
echo ""

CPU="-accel tcg,thread=multi -cpu cortex-a72 -smp 4"

qemu-system-aarch64 -M virt $CPU -m 2G \
    -bios tools/firmware/edk2-aarch64-code.fd \
    -drive file="$QCOW2",format=qcow2,id=hd0,if=none \
    -device virtio-blk-pci,drive=hd0 \
    -device virtio-9p-pci,fsdev=host0,mount_tag=hostshare \
    -fsdev local,id=host0,path="$SHARE",security_model=none \
    -device qemu-xhci -device usb-kbd \
    -serial tcp::${SERIAL_PORT},server,nowait \
    -monitor tcp::${MONITOR_PORT},server,nowait \
    -gdb tcp::${GDB_PORT} \
    -S \
    -daemonize

echo "QEMU PID: $(pgrep -f "serial tcp::${SERIAL_PORT}")"
