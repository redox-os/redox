#!/bin/bash
# Quick snapshot management for qcow2 development images
# Usage: ./snapshot.sh [save|list|load|delete|reset] [name]
set -e

cd "$(dirname "$0")"
ROOT="$(pwd)"
QCOW2="${QCOW2:-$ROOT/build/aarch64/dev.qcow2}"
BASE_ISO="$ROOT/build/aarch64/pure-rust.iso"
STATEFILE="${QCOW2}.state"

if [[ ! -f "$QCOW2" ]]; then
    echo "Error: qcow2 not found: $QCOW2"
    echo "Run ./run-dev.sh first to create it"
    exit 1
fi

case "$1" in
    save|s)
        NAME="${2:-snap-$(date +%H%M%S)}"
        # If snapshot exists, rename it to name.bak first
        if qemu-img snapshot -l "$QCOW2" 2>/dev/null | grep -qw "$NAME"; then
            echo "Renaming existing '$NAME' to '${NAME}.bak'..."
            qemu-img snapshot -d "${NAME}.bak" "$QCOW2" 2>/dev/null || true
            # Save current state temporarily
            qemu-img snapshot -c "_tmp_save_" "$QCOW2"
            # Load old snapshot, save as .bak
            qemu-img snapshot -a "$NAME" "$QCOW2"
            qemu-img snapshot -c "${NAME}.bak" "$QCOW2"
            # Restore current state and clean up
            qemu-img snapshot -a "_tmp_save_" "$QCOW2"
            qemu-img snapshot -d "_tmp_save_" "$QCOW2"
            qemu-img snapshot -d "$NAME" "$QCOW2"
        fi
        qemu-img snapshot -c "$NAME" "$QCOW2"
        echo "$NAME" > "$STATEFILE"
        echo "Saved snapshot: $NAME"
        ;;
    list|l|ls|status|st)
        CURRENT="(unknown)"
        [[ -f "$STATEFILE" ]] && CURRENT="$(cat "$STATEFILE")"
        echo "Current: $CURRENT"
        echo "Snapshots in $QCOW2:"
        qemu-img snapshot -l "$QCOW2" 2>/dev/null || echo "  (none)"
        ;;
    load|restore|r)
        if [[ -z "$2" ]]; then
            echo "Usage: $0 load <snapshot-name>"
            echo "Available snapshots:"
            qemu-img snapshot -l "$QCOW2"
            exit 1
        fi
        qemu-img snapshot -a "$2" "$QCOW2"
        echo "$2" > "$STATEFILE"
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
            echo "base" > "$STATEFILE"
            echo "Reset to 'base' snapshot (other snapshots preserved)"
        else
            echo "No 'base' snapshot found. Create one with: $0 save base"
            echo "Or use '$0 nuke' to recreate from ISO (destroys all snapshots)"
            exit 1
        fi
        ;;
    nuke)
        echo "⚠️ ⚡️ 🚨 ⚡︎ ⚠️ ⚡️ 🚨 ⚡︎"
        echo "Recreating qcow2 from base ISO (ALL SNAPSHOTS WILL BE LOST)..."
read -p "Are you sure? Type 'yes' to continue: " CONFIRM
if [[ "$CONFIRM" != "yes" ]]; then
    echo "Aborted."
    exit 1
fi
        rm -f "$QCOW2" "$STATEFILE"
        qemu-img create -f qcow2 -b "$BASE_ISO" -F raw "$QCOW2"
        echo "(iso)" > "$STATEFILE"
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
        echo "  list/status   - Show current + all snapshots"
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
