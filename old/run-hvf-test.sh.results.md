=== Testing: cortex-a72 (baseline) ===
Options: -M virt -cpu cortex-a72
✅ OK (or timed out normally)

=== Testing: HVF + GIC v2 ===
Options: -M virt,gic-version=2 -accel hvf -cpu host
✅ OK (or timed out normally)

=== Testing: HVF + no virtualization ext ===
Options: -M virt,virtualization=off -accel hvf -cpu host
✅ OK (or timed out normally)

=== Testing: HVF + no highmem ===
Options: -M virt,highmem=off -accel hvf -cpu host
✅ OK (or timed out normally)

=== Testing: HVF + GIC v2 + no virt ext ===
Options: -M virt,gic-version=2,virtualization=off -accel hvf -cpu host
✅ OK (or timed out normally)

