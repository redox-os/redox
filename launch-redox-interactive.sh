#!/bin/bash
# Launch Redox OS interactively in the terminal

set -e

FIRMWARE="/opt/homebrew/opt/qemu/share/qemu/edk2-aarch64-code.fd"
DISK="/Users/me/.redox-vms/test.img"

if [ ! -f "$DISK" ]; then
    echo "Disk image not found. Run 'redox-vm launch --name test --mem 2G' first"
    exit 1
fi

echo "Launching Redox OS in interactive mode..."
echo "Press ENTER at bootloader to use default resolution"
echo "To exit: Ctrl-A then X"
echo ""

qemu-system-aarch64 \
    -machine virt -cpu max -accel hvf \
    -smp 4 -m 2048 \
    -drive "if=pflash,format=raw,unit=0,file=$FIRMWARE,readonly=on" \
    -drive "file=$DISK,format=raw" \
    -device qemu-xhci -device usb-kbd -device usb-tablet \
    -device e1000,netdev=net0 \
    -netdev user,id=net0,hostfwd=tcp::8022-:22 \
    -nographic \
    -serial mon:stdio \
    -name "Redox OS Interactive"
