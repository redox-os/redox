#!/bin/bash
# Run Redox OS aarch64 with Cranelift-compiled kernel in QEMU
set -e

REDOX_DIR="$(cd "$(dirname "$0")" && pwd)"
ISO="$REDOX_DIR/build/aarch64/desktop/redox-live.iso"
KERNEL_SRC="$REDOX_DIR/recipes/core/kernel/source/target/aarch64-redox-none/release/kernel"
MOUNT_POINT="/tmp/redoxfs-mount"
SERIAL_LOG="/tmp/qemu-aarch64-serial.log"
MONITOR_SOCK="/tmp/qemu-aarch64-monitor.sock"

# Check prerequisites
if [[ ! -f "$ISO" ]]; then
    echo "Error: ISO not found at $ISO"
    echo "Build with: make ARCH=aarch64 live"
    exit 1
fi

if [[ ! -f "$KERNEL_SRC" ]]; then
    echo "Error: Cranelift kernel not found at $KERNEL_SRC"
    echo "Build with: cd recipes/core/kernel/source && cargo +nightly build --target aarch64-redox-none.json --release"
    exit 1
fi

# Replace kernel if requested
if [[ "$1" == "--inject" || "$1" == "-i" ]]; then
    echo "Injecting Cranelift kernel into ISO..."

    mkdir -p "$MOUNT_POINT"

    # Unmount if already mounted
    umount "$MOUNT_POINT" 2>/dev/null || true

    # Mount RedoxFS
    "$REDOX_DIR/build/fstools/bin/redoxfs" "$ISO" "$MOUNT_POINT"
    sleep 2

    # Strip and copy kernel
    KERNEL_STRIPPED="/tmp/kernel-aarch64-stripped"
    llvm-strip -o "$KERNEL_STRIPPED" "$KERNEL_SRC"
    cp "$KERNEL_STRIPPED" "$MOUNT_POINT/boot/kernel"

    echo "Kernel replaced: $(ls -lh "$MOUNT_POINT/boot/kernel" | awk '{print $5}')"

    # Unmount
    umount "$MOUNT_POINT"
    echo "ISO updated with Cranelift kernel"
fi

# Clean up old QEMU instances
pkill -9 -f "qemu-system-aarch64.*redox-live.iso" 2>/dev/null || true
rm -f "$SERIAL_LOG" "$MONITOR_SOCK"

echo "Starting QEMU aarch64..."
echo "Serial log: $SERIAL_LOG"
echo "Monitor: echo 'sendkey ret' | nc -U $MONITOR_SOCK"

qemu-system-aarch64 -M virt -cpu cortex-a72 -m 2G \
    -device ramfb -device qemu-xhci -device usb-kbd -device usb-tablet \
    -bios /opt/homebrew/opt/qemu/share/qemu/edk2-aarch64-code.fd \
    -drive file="$ISO",format=raw,if=virtio \
    -serial file:"$SERIAL_LOG" \
    -monitor unix:"$MONITOR_SOCK",server,nowait \
    -display none &

QEMU_PID=$!
echo "QEMU PID: $QEMU_PID"

# Wait for bootloader menu
sleep 6

# Send Enter to select resolution
for i in 1 2; do
    echo "sendkey ret" | nc -U "$MONITOR_SOCK" >/dev/null 2>&1
    sleep 1
done

echo "Waiting for boot (30s)..."
sleep 30

echo "=== Boot Log (last 50 lines) ==="
tail -50 "$SERIAL_LOG" | grep -v '^\[2J\|^\[0m'

echo ""
echo "QEMU running in background (PID: $QEMU_PID)"
echo "View full log: tail -f $SERIAL_LOG"
echo "Stop: kill $QEMU_PID"
