

### HVF Investigation Status

The icache fix was added to RMM:
- `sync_icache()` method in Arch trait (default no-op)
- aarch64 implementation using `dc cvau` + `ic ivau`
- Calls in mmap/mprotect for PROT_EXEC

Testing results:
- Original ISO with HVF: boots to login, crashes on userspace interaction ("Lacks grant")
- New kernel with HVF: crashes earlier at kernel init (data abort at 0x11000)

The new crash may be due to:
1. Different compiler/toolchain settings between original and Cranelift builds
2. Missing configuration options
3. Timing-sensitive code that behaves differently

Further investigation needed to match original kernel build settings.

## Important Findings

### Linker Script Fix is Confirmed Working

Kernels built with the linker script fix:
- Boot to login prompt ✅
- Show same "Lacks grant" behavior as original ISO ✅
- Same 2.48MB size when stripped ✅

### icache Fix Status

The icache fix (sync_icache with dc cvau + ic ivau) causes an early kernel crash:
- Crash at kernel init (data abort at 0x11000)
- Likely due to inline assembly incompatibility with Cranelift codegen
- The fix concept is correct but implementation may need adjustment for Cranelift

### HVF Crash Behavior

The "Lacks grant" crash happens in userspace programs (fbcond, pcid-spawner, inputd, usbhidd) with:
- Exception type: synchronous exception, not SVC
- ESR_EL1 indicates instruction cache/data abort issues

This suggests the original kernel also needs icache fixes, but the fix needs to be compatible with Cranelift codegen.

### Possible Next Steps

1. Try building icache fix with LLVM instead of Cranelift
2. Use a simpler icache sync approach that doesn't use inline asm
3. Investigate Cranelift's inline asm support for ARM64
