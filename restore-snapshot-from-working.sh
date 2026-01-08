#!/bin/bash
# Restore dev state from the .works backup copies
set -e
cd "$(dirname "$0")"

WORKS_ISO="build/aarch64/pure-rust-works.iso"
WORKS_QCOW2="build/aarch64/dev.qcow2.works"

if [[ ! -f "$WORKS_ISO" ]] || [[ ! -f "$WORKS_QCOW2" ]]; then
    echo "Error: .works backups not found"
    echo "  Expected: $WORKS_ISO"
    echo "           $WORKS_QCOW2"
    exit 1
fi

echo "Restoring dev state from .works backups..."
cp "$WORKS_ISO" build/aarch64/pure-rust.iso
cp "$WORKS_QCOW2" build/aarch64/dev.qcow2
qemu-img rebase -u -b pure-rust.iso -F raw build/aarch64/dev.qcow2

echo "Setting restored state as 'base' snapshot..."
./snapshot.sh save base

echo "Done. Dev environment restored from .works backup."
