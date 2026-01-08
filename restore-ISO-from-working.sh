#!/bin/bash
# Restore dev state from the .works backup copies
# WARNING: This overwrites pure-rust.iso and dev.qcow2!
set -e
cd "$(dirname "$0")"

WORKS_ISO="build/aarch64/pure-rust-works.iso"
WORKS_QCOW2="build/aarch64/dev.qcow2.works"
DEV_QCOW2="build/aarch64/dev.qcow2"
DEV_ISO="build/aarch64/pure-rust.iso"
BACKUP_QCOW2="build/aarch64/dev.qcow2.backup-$(date +%Y%m%d-%H%M%S)"

if [[ ! -f "$WORKS_ISO" ]] || [[ ! -f "$WORKS_QCOW2" ]]; then
    echo "Error: .works backups not found"
    echo "  Expected: $WORKS_ISO"
    echo "           $WORKS_QCOW2"
    exit 1
fi

echo "========================================="
echo "WARNING: This will OVERWRITE:"
echo "  - $DEV_ISO"
echo "  - $DEV_QCOW2 (all snapshots lost!)"
echo ""
echo "Current qcow2 will be backed up as standalone image:"
echo "  - $BACKUP_QCOW2"
echo "  (flattened - works even if ISO is deleted)"
echo "========================================="
echo ""
read -p "Are you sure? Type 'yes' to continue: " CONFIRM

if [[ "$CONFIRM" != "yes" ]]; then
    echo "Aborted."
    exit 1
fi

echo ""
echo "Flattening current qcow2 to standalone backup..."
qemu-img convert -O qcow2 "$DEV_QCOW2" "$BACKUP_QCOW2"
echo "Backup saved: $BACKUP_QCOW2"

echo ""
echo "Restoring dev state from .works backups..."
cp "$WORKS_ISO" "$DEV_ISO"
cp "$WORKS_QCOW2" "$DEV_QCOW2"
qemu-img rebase -u -b pure-rust.iso -F raw "$DEV_QCOW2"

echo ""
echo "Setting restored state as 'base' snapshot..."
./snapshot.sh save base

echo ""
echo "Done. Dev environment restored from .works backup."
echo "Old state preserved in: $BACKUP_QCOW2"
