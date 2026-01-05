#!/bin/bash
ISO="/opt/other/redox/build/aarch64/server-official.iso"
SERIAL_LOG="/tmp/qemu-official-serial.log"
MONITOR_SOCK="/tmp/qemu-official-monitor.sock"

rm -f "$SERIAL_LOG" "$MONITOR_SOCK"

qemu-system-aarch64 -M virt -cpu cortex-a72 -m 2G \
    -device ramfb -device qemu-xhci -device usb-kbd -device usb-tablet \
    -bios /opt/homebrew/opt/qemu/share/qemu/edk2-aarch64-code.fd \
    -drive file="$ISO",format=raw,if=virtio \
    -serial file:"$SERIAL_LOG" \
    -monitor unix:"$MONITOR_SOCK",server,nowait \
    -display none &

QEMU_PID=$!
echo "QEMU PID: $QEMU_PID"

sleep 8
echo "sendkey ret" | nc -U "$MONITOR_SOCK" 2>/dev/null
sleep 2
echo "sendkey ret" | nc -U "$MONITOR_SOCK" 2>/dev/null
sleep 35

echo "=== Boot log ==="
tail -60 "$SERIAL_LOG" | grep -v '^\[2J'
