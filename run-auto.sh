#!/bin/bash
# Run Redox with auto-login via expect
set -e

ISO="${1:-build/aarch64/server-cranelift.iso}"
SHARE="${2:-/tmp/9p-share}"

mkdir -p "$SHARE"
[ -f "$SHARE/test.txt" ] || echo "Hello from host filesystem via virtio-9p!" > "$SHARE/test.txt"

echo "Starting Redox with AUTO-LOGIN"
echo "9p share at: $SHARE"
echo "Press Ctrl-A X to exit QEMU"
echo

# Check if expect is available
if command -v expect &>/dev/null; then
    expect -c "
        set timeout -1
        spawn qemu-system-aarch64 -M virt -cpu cortex-a72 -m 2G \
            -bios tools/firmware/edk2-aarch64-code.fd \
            -drive file=$ISO,format=raw,id=hd0,if=none \
            -device virtio-blk-pci,drive=hd0 \
            -device virtio-9p-pci,fsdev=host0,mount_tag=hostshare \
            -fsdev local,id=host0,path=$SHARE,security_model=none \
            -device qemu-xhci -device usb-kbd \
            -nographic

        # Wait for login prompt and auto-login as root
        expect {
            \"login:\" { send \"root\r\" }
            timeout { }
        }
        expect {
            \"Password:\" { send \"password\r\" }
            \"#\" { }
            timeout { }
        }

        # Hand control to user
        interact
    "
else
    echo "Note: 'expect' not found - manual login required (root/password)"
    exec qemu-system-aarch64 -M virt -cpu cortex-a72 -m 2G \
        -bios tools/firmware/edk2-aarch64-code.fd \
        -drive file="$ISO",format=raw,id=hd0,if=none \
        -device virtio-blk-pci,drive=hd0 \
        -device virtio-9p-pci,fsdev=host0,mount_tag=hostshare \
        -fsdev local,id=host0,path="$SHARE",security_model=none \
        -device qemu-xhci -device usb-kbd \
        -nographic
fi
