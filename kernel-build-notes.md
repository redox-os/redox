# Kernel Build Notes for aarch64 Cranelift

## Critical Finding: Missing Linker Script

The `build-cranelift.sh` script was missing the linker script for aarch64 kernel builds.

### Problem

The kernel was building to ~4.5MB but only had 332 bytes of actual LOAD segment (code):
```
ProgramHeader {
  Type: PT_LOAD (0x1)
  FileSize: 332   # <-- This should be ~1.7MB!
}
```

### Root Cause

In `build-cranelift.sh`, the linker script was only added for x86_64:
```bash
# x86_64 needs linker script
if [ "$ARCH" = "x86_64" ]; then
    kernel_rustflags="$kernel_rustflags -C relocation-model=static -C link-arg=-Tlinkers/x86_64.ld"
fi
```

### Fix

Added aarch64 linker script support:
```bash
# All architectures need linker script
if [ "$ARCH" = "x86_64" ]; then
    kernel_rustflags="$kernel_rustflags -C relocation-model=static -C link-arg=-Tlinkers/x86_64.ld"
elif [ "$ARCH" = "aarch64" ]; then
    kernel_rustflags="$kernel_rustflags -C relocation-model=static -C link-arg=-Tlinkers/aarch64.ld"
fi
```

### Verification

After fix, kernel has proper structure:
- Code segment: ~1.7MB (PF_R|PF_X)
- Data segments with proper sizes
- Kernel size after stripping: ~2.48MB (matches original)

The linker script at `linkers/aarch64.ld` properly maps:
- `.text` section at `KERNEL_OFFSET = 0xFFFFFF0000000000`
- `.rodata`, `.data`, `.bss` sections
- Discards debug info (`.eh_frame`, `.comment`, etc.)

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
