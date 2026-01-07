#!/bin/bash
# Inject new initfs into existing ISO

set -e

ISO="${1:-/opt/other/redox/build/aarch64/server-cranelift.iso}"
INITFS="/tmp/initfs-cranelift.img"
REDOXFS="/opt/other/redox/build/fstools/bin/redoxfs"
MOUNT="/tmp/redoxfs-inject-mount"

if [[ ! -f "$ISO" ]]; then
    echo "Error: ISO not found at $ISO"
    exit 1
fi

if [[ ! -f "$INITFS" ]]; then
    echo "Error: initfs not found at $INITFS"
    exit 1
fi

echo "=== Mounting ISO ==="
mkdir -p "$MOUNT"
"$REDOXFS" "$ISO" "$MOUNT"
sleep 2

echo "=== Current boot directory ==="
ls -la "$MOUNT/boot/"

echo "=== Replacing initfs ==="
cp "$INITFS" "$MOUNT/boot/initfs"
sync

echo "=== New initfs size ==="
ls -la "$MOUNT/boot/initfs"

echo "=== Unmounting ==="
umount "$MOUNT" 2>/dev/null || fusermount -u "$MOUNT" 2>/dev/null || diskutil unmount "$MOUNT" 2>/dev/null
rmdir "$MOUNT"

echo "=== Done! ISO updated ==="
