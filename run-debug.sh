#!/bin/bash
# Run Redox for parallel debugging - uses different ports to allow multiple instances
set -e

ISO="${1:-build/aarch64/server-cranelift.iso}"
SHARE="${2:-/tmp/9p-debug}"
INSTANCE="${3:-1}"

# Calculate unique ports based on instance
SERIAL_PORT=$((4440 + INSTANCE))
MONITOR_PORT=$((4450 + INSTANCE))
GDB_PORT=$((1234 + INSTANCE))

mkdir -p "$SHARE"

echo "Starting Redox DEBUG instance #$INSTANCE"
echo "9p share: $SHARE"
echo "Serial port: localhost:$SERIAL_PORT (telnet to connect)"
echo "Monitor port: localhost:$MONITOR_PORT"
echo "GDB port: localhost:$GDB_PORT"
echo ""
echo "Connect with: telnet localhost $SERIAL_PORT"
echo "Or: socat -,rawer tcp:localhost:$SERIAL_PORT"
echo ""

qemu-system-aarch64 -M virt -cpu cortex-a72 -m 2G \
    -bios tools/firmware/edk2-aarch64-code.fd \
    -drive file="$ISO",format=raw,id=hd0,if=none \
    -device virtio-blk-pci,drive=hd0 \
    -device virtio-9p-pci,fsdev=host0,mount_tag=hostshare \
    -fsdev local,id=host0,path="$SHARE",security_model=none \
    -device qemu-xhci -device usb-kbd \
    -serial tcp::${SERIAL_PORT},server,nowait \
    -monitor tcp::${MONITOR_PORT},server,nowait \
    -gdb tcp::${GDB_PORT} \
    -S \
    -daemonize

echo "QEMU running in background (instance #$INSTANCE)"
echo "PID: $(pgrep -f "serial tcp::${SERIAL_PORT}")"
