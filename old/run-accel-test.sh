#!/bin/bash
# Test different QEMU acceleration options for aarch64 Redox
# Goal: find if any HVF/accel combo works without crashes
set -e

ISO="${1:-build/aarch64/server-cranelift.iso.ok.bak}"
SHARE="/tmp/9p-accel-test"
mkdir -p "$SHARE"

echo "Testing QEMU acceleration options with: $ISO"
echo "Each test runs for 30s - watch for 'Lacks grant' or exception errors"
echo

run_test() {
    local name="$1"
    local opts="$2"
    echo "=== Testing: $name ==="
    echo "Options: $opts"
    timeout 20 qemu-system-aarch64 -M virt $opts -m 2G \
        -bios tools/firmware/edk2-aarch64-code.fd \
        -drive file="$ISO",format=raw,id=hd0,if=none \
        -device virtio-blk-pci,drive=hd0 \
        -device virtio-9p-pci,fsdev=host0,mount_tag=hostshare \
        -fsdev local,id=host0,path="$SHARE",security_model=none \
        -device qemu-xhci -device usb-kbd \
        -nographic 2>&1 | tee /tmp/qemu-test.log | tail -30

    if grep -q "Lacks grant\|FATAL:\|UNHANDLED EXCEPTION" /tmp/qemu-test.log; then
        echo "❌ CRASHED"
    else
        echo "✅ No crash detected (may have timed out normally)"
    fi
    echo
    sleep 2
}

# Test 1: Baseline (known working)
# run_test "cortex-a72 (baseline)" "-cpu cortex-a72"

# Test 2: HVF with host CPU (known broken)
# run_test "HVF + host CPU" "-accel hvf -cpu host"

# Test 4: HVF with max CPU
run_test "HVF + max CPU" "-accel hvf -cpu max"

run_test "HVF" "-accel hvf"

run_test "host CPU" "-cpu host"

# Test 3: HVF with cortex-a72
run_test "HVF + cortex-a72" "-accel hvf -cpu cortex-a72"

# Test 5: TCG explicit (software emulation)
run_test "TCG explicit" "-accel tcg -cpu cortex-a72"

# Test 6: TCG with threading
run_test "TCG multi-thread" "-accel tcg,thread=multi -cpu cortex-a72"

echo "All tests complete!"
