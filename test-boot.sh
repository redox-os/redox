#!/bin/bash
rm -f /tmp/qemu-serial.log
touch /tmp/qemu-serial.log

qemu-system-aarch64 -M virt -cpu cortex-a72 -m 2G \
  -device ramfb -device qemu-xhci -device usb-kbd -device usb-tablet \
  -bios /opt/other/qemu-uefi/QEMU_EFI.fd \
  -drive file=/opt/other/redox/build/aarch64/desktop/redox-live.iso,format=raw,if=virtio \
  -serial file:/tmp/qemu-serial.log \
  -display none &
QEMU_PID=$!

sleep 25

echo "=== ipcd/daemon related output ==="
grep -i "ipcd\|daemon" /tmp/qemu-serial.log | head -30

echo ""
echo "=== Last 60 lines of boot log ==="
tail -60 /tmp/qemu-serial.log

kill $QEMU_PID 2>/dev/null
