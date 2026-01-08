#!/bin/bash
# Snapshot current dev state to the .works copies for parallel use
set -e
cd "$(dirname "$0")"

echo "Setting snapshot as new 'base'"
./snapshot.sh save base # is always safe - overwrites keep the previous version as base.bak

echo "Snapshotting current dev state to .works copies..."
cp build/aarch64/pure-rust.iso build/aarch64/pure-rust-works.iso
cp build/aarch64/dev.qcow2 build/aarch64/dev.qcow2.works
qemu-img rebase -u -b pure-rust-works.iso -F raw build/aarch64/dev.qcow2.works
echo "Done. run-console.sh now uses the updated .works snapshot."
