#!/bin/bash
# Test HVF acceleration options to find one that works
set -e

ISO="${1:-build/aarch64/server-cranelift.iso.ok.bak}"
SHARE="/tmp/9p-hvf-test"
mkdir -p "$SHARE"

run_test() {
    local name="$1"
    shift
    echo "=== Testing: $name ==="
    echo "Options: $@"
    qemu-system-aarch64 "$@" -m 2G \
        -bios tools/firmware/edk2-aarch64-code.fd \
        -drive file="$ISO",format=raw,id=hd0,if=none \
        -device virtio-blk-pci,drive=hd0 \
        -device virtio-9p-pci,fsdev=host0,mount_tag=hostshare \
        -fsdev local,id=host0,path="$SHARE",security_model=none \
        -device qemu-xhci -device usb-kbd \
        -nographic 

    if grep -q "Lacks grant\|FATAL:\|UNHANDLED EXCEPTION" /tmp/qemu-hvf.log; then
        echo "❌ CRASHED"
    else
        echo "✅ OK (or timed out normally)"
    fi
    echo; sleep 2
}

# Baseline - known working
run_test "cortex-a72 (baseline)" -M virt -cpu cortex-a72

# HVF tests - different combos
# run_test "HVF + GIC v2" -M virt,gic-version=2 -accel hvf -cpu host
# run_test "HVF + no virtualization ext" -M virt,virtualization=off -accel hvf -cpu host
# run_test "HVF + no highmem" -M virt,highmem=off -accel hvf -cpu host
# run_test "HVF + GIC v2 + no virt ext" -M virt,gic-version=2,virtualization=off -accel hvf -cpu host

echo "Done! Check which options avoid the crash."
