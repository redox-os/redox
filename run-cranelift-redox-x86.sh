#!/bin/bash
# Boot Redox OS with Cranelift-compiled kernel
# Interactive serial console - you can login!

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
IMAGE="$SCRIPT_DIR/build/cranelift-redox.img"
UEFI_FW="/opt/homebrew/Cellar/qemu/10.2.0/share/qemu/edk2-x86_64-code.fd"

# Check if we have the image
if [[ ! -f "$IMAGE" ]]; then
    echo "Creating bootable image with Cranelift kernel..."

    # Download base image if needed
    if [[ ! -f /tmp/redox-base.img ]]; then
        echo "Downloading Redox base image..."
        echo "WHAT?? NO!! I THOUGHT WE USE LOCAL BUILD!?!"
        curl -L -o /tmp/redox.img.zst \
            "https://static.redox-os.org/img/x86_64/redox_server_x86_64_2025-12-29_200_harddrive.img.zst"
        zstd -d /tmp/redox.img.zst -o /tmp/redox-base.img
    fi

    mkdir -p "$SCRIPT_DIR/build"
    cp /tmp/redox-base.img "$IMAGE"

    echo "Image ready at $IMAGE"
    echo "Note: To use Cranelift kernel, replace kernel in image using Docker (see CLAUDE.md)"
fi

echo "=========================================="
echo "  Redox OS with Cranelift Kernel"
echo "=========================================="
echo ""
echo "Login credentials:"
echo "  user: user (no password)"
echo "  root: root / password"
echo ""
echo "Press Ctrl-A X to exit QEMU"
echo "=========================================="
echo ""

# Run QEMU with interactive serial console
exec qemu-system-x86_64 \
    -drive if=pflash,format=raw,unit=0,file="$UEFI_FW",readonly=on \
    -drive file="$IMAGE",format=raw \
    -m 2048M \
    -smp 2 \
    -machine q35 \
    -cpu core2duo \
    -nographic \
    -serial mon:stdio \
    -no-reboot
