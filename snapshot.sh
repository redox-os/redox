#!/bin/bash
# Quick snapshot management for qcow2 development images
# Usage: ./snapshot.sh [save|list|load|delete|reset] [name]
set -e

cd "$(dirname "$0")"
ROOT="$(pwd)"
QCOW2="${QCOW2:-$ROOT/build/aarch64/dev.qcow2}"
BASE_ISO="$ROOT/build/aarch64/pure-rust.iso"

if [[ ! -f "$QCOW2" ]]; then
    echo "Error: qcow2 not found: $QCOW2"
    echo "Run ./run-dev.sh first to create it"
    exit 1
fi

case "$1" in
    save|s)
        NAME="${2:-snap-$(date +%H%M%S)}"
        qemu-img snapshot -c "$NAME" "$QCOW2"
        echo "Saved snapshot: $NAME"
        ;;
    list|l|ls)
        echo "Snapshots in $QCOW2:"
        qemu-img snapshot -l "$QCOW2"
        ;;
    load|restore|r)
        if [[ -z "$2" ]]; then
            echo "Usage: $0 load <snapshot-name>"
            echo "Available snapshots:"
            qemu-img snapshot -l "$QCOW2"
            exit 1
        fi
        qemu-img snapshot -a "$2" "$QCOW2"
        echo "Restored snapshot: $2"
        ;;
    delete|d|rm)
        if [[ -z "$2" ]]; then
            echo "Usage: $0 delete <snapshot-name>"
            exit 1
        fi
        qemu-img snapshot -d "$2" "$QCOW2"
        echo "Deleted snapshot: $2"
        ;;
    reset)
        # Try to load "base" snapshot if it exists, otherwise warn
        if qemu-img snapshot -l "$QCOW2" 2>/dev/null | grep -q "base"; then
            qemu-img snapshot -a "base" "$QCOW2"
            echo "Reset to 'base' snapshot (other snapshots preserved)"
        else
            echo "No 'base' snapshot found. Create one with: $0 save base"
            echo "Or use '$0 nuke' to recreate from ISO (destroys all snapshots)"
            exit 1
        fi
        ;;
    nuke)
        echo "Recreating qcow2 from base ISO (all snapshots will be lost)..."
        rm -f "$QCOW2"
        qemu-img create -f qcow2 -b "$BASE_ISO" -F raw "$QCOW2"
        echo "Nuked - clean slate from ISO"
        ;;
    info|i)
        qemu-img info "$QCOW2"
        ;;
    *)
        echo "Usage: $0 <command> [args]"
        echo ""
        echo "Commands:"
        echo "  save [name]   - Create snapshot (default: snap-HHMMSS)"
        echo "  list          - List all snapshots"
        echo "  load <name>   - Restore to snapshot"
        echo "  delete <name> - Delete snapshot"
        echo "  reset         - Load 'base' snapshot (preserves other snapshots)"
        echo "  nuke          - Recreate qcow2 from ISO (destroys ALL snapshots)"
        echo "  info          - Show qcow2 info"
        echo ""
        echo "Examples:"
        echo "  $0 save before-test   # Save current state"
        echo "  $0 list               # Show snapshots"
        echo "  $0 load before-test   # Rollback"
        ;;
esac
