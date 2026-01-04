# Redox OS Development Notes

## Cranelift Backend Experiment - SUCCESS! 🎉

### Breakthrough: Kernel builds with Cranelift!

On 2026-01-04, we successfully compiled the Redox kernel using Cranelift instead of LLVM.

**Fork with fixes:** https://github.com/pannous/rustc_codegen_cranelift

### Key Changes Made

1. **Implemented `sym` operand support** (commit a7883ae1)
   - Added Mach-O underscore prefix for symbol names
   - Created wrapper functions for global_asm sym operands

2. **Fixed duplicate symbol errors** (commit 2106ade4)
   - Made wrapper names unique by including CGU name

3. **Fixed kernel Intel syntax** (commit ff9ac52c in kernel repo)
   - Changed `int $3` to `int 3` for Cranelift compatibility

### Working Build Command

```bash
cd recipes/core/kernel/source

# Use nightly-2026-01-02 to match Cranelift backend
DYLD_LIBRARY_PATH=/Users/me/.rustup/toolchains/nightly-2026-01-02-aarch64-apple-darwin/lib \
RUSTFLAGS="-Zcodegen-backend=/opt/other/rustc_codegen_cranelift/dist/lib/librustc_codegen_cranelift.dylib" \
cargo +nightly-2026-01-02 build \
  --target x86_64-unknown-none \
  --release \
  -Z build-std=core,alloc \
  -Zbuild-std-features=compiler-builtins-mem,compiler_builtins/no-f16-f128
```

### Result

```
kernel: ELF 64-bit LSB pie executable, x86-64, version 1 (SYSV)
Size: 4.4 MB
```

### Build Cranelift Backend

```bash
cd /opt/other/rustc_codegen_cranelift
./y.sh prepare
./y.sh build --sysroot clif
```

### relibc Status

**Blocker:** Cranelift doesn't support variadic functions yet.

relibc defines many variadic C functions (`printf`, `scanf`, `syslog`, etc.):
```rust
pub unsafe extern "C" fn syslog(priority: c_int, message: *const c_char, mut __valist: ...) {
```

Error: `Defining variadic functions is not yet supported by Cranelift`

VaList API was updated for nightly-2026-01-02 (commit f339c31f in relibc source).

### Mac (aarch64) Testing

Cranelift works excellently on Mac for normal Rust code:

| Feature | Status |
|---------|--------|
| std library | ✅ |
| Inline asm | ✅ |
| global_asm + sym | ✅ |
| Threading, Arc, Mutex | ✅ |
| Serde + serde_json | ✅ |
| Tokio async runtime | ✅ |

The only blocker is *defining* variadic functions (relibc needs this).

### Next Steps

- Monitor Cranelift variadic function support
- Test kernel functionality in QEMU
- Contribute `sym` operand support upstream

### Historical Context

Initial blockers (now resolved):
- `sym` operands in inline asm - **FIXED** in fork
- `int $3` vs `int 3` syntax - **FIXED** in kernel
- Duplicate wrapper symbols - **FIXED** in fork

### QEMU Boot Test - SUCCESS! 🎉

On 2026-01-04, the Cranelift-compiled kernel was successfully booted in QEMU.

**Boot Log Highlights:**
```
kernel: 8/8 MiB (loaded Cranelift kernel)
kernel::arch::x86_shared::start:INFO -- Redox OS starting...
kernel::startup::memory:INFO -- Memory: 1979 MB
Framebuffer 1280x800 stride 1280 at 80000000
vesad: 1280x800 stride 1280 at 0x80000000
ahcid: SATA QEMU HARDDISK 512 MB detected
redox login: (reached login prompt!)
```

**Critical Fix for Boot:**
The linker script must be explicitly passed via RUSTFLAGS:
```bash
RUSTFLAGS="-Zcodegen-backend=.../librustc_codegen_cranelift.dylib \
           -C relocation-model=static \
           -C link-arg=-Tlinkers/x86_64.ld"
```

Without the linker script, the kernel had no .text section and entry point was 0x0.

**Tested Configuration:**
- QEMU x86_64 with UEFI (edk2-x86_64-code.fd)
- 2GB RAM, 2 CPUs, Q35 machine
- Disk: Pre-built Redox server image with kernel replaced

### relibc Compiled with Cranelift - SUCCESS! 🎉

On 2026-01-04, relibc (Redox's C library) was compiled using Cranelift.

**Key Change:** Your commit `a86211e4` added variadic function support!

**Build Command:**
```bash
cd recipes/core/relibc/source

DYLD_LIBRARY_PATH=~/.rustup/toolchains/nightly-2026-01-02-aarch64-apple-darwin/lib \
RUSTFLAGS="-Zcodegen-backend=/opt/other/rustc_codegen_cranelift/dist/lib/librustc_codegen_cranelift.dylib" \
cargo +nightly-2026-01-02 build \
  --target x86_64-unknown-redox \
  --release \
  -Z build-std=core,alloc \
  -Zbuild-std-features=compiler_builtins/no-f16-f128
```

**Result:**
```
librelibc.a: 16 MB
```

This means both the kernel AND the C library can now be compiled with a pure Rust toolchain!

### aarch64 Kernel Build - SUCCESS! 🎉

On 2026-01-04, the Redox kernel was compiled for aarch64 using Cranelift!

**Key Fix:** Added ELF binary format detection for freestanding targets (commit b0aaea74 in Cranelift fork).

The `aarch64-unknown-none` target doesn't specify a binary format in the target triple.
Fixed by detecting "m:e" in data-layout string → ELF format.

**Custom Target Required:**
Created `aarch64-redox-none.json` with `max-atomic-width: 64` (Cranelift doesn't support 128-bit atomics on aarch64 yet).

**Build Command:**
```bash
cd recipes/core/kernel/source

DYLD_LIBRARY_PATH=~/.rustup/toolchains/nightly-2026-01-02-aarch64-apple-darwin/lib \
RUSTFLAGS="-Zcodegen-backend=/opt/other/rustc_codegen_cranelift/dist/lib/librustc_codegen_cranelift.dylib" \
cargo +nightly-2026-01-02 build \
  --target aarch64-redox-none.json \
  --release \
  -Z build-std=core,alloc \
  -Zbuild-std-features=compiler-builtins-mem,compiler_builtins/no-f16-f128
```

**Result:**
```
kernel: ELF 64-bit LSB executable, ARM aarch64, version 1 (SYSV)
Size: 4.5 MB
```

**QEMU aarch64 Testing:**
Created minimal test binary to verify Cranelift aarch64 codegen:
- Arithmetic operations ✅
- Recursive function calls (Fibonacci) ✅
- Stack arrays and iteration ✅

```
=== aarch64 Cranelift Test ===
Arithmetic: 42 + 100 = 000000000000008e
Fibonacci(10) = 0000000000000037 (55)
Array sum = 000000000000000f (15)
=== All tests passed! ===
```

### relibc aarch64 Build - SUCCESS! 🎉

On 2026-01-04, relibc was compiled for aarch64 using Cranelift!

**Custom Target:** Created `aarch64-unknown-redox-clif.json` with `max-atomic-width: 64`

**Build Command:**
```bash
cd recipes/core/relibc/source

DYLD_LIBRARY_PATH=~/.rustup/toolchains/nightly-2026-01-02-aarch64-apple-darwin/lib \
RUSTFLAGS="-Zcodegen-backend=/opt/other/rustc_codegen_cranelift/dist/lib/librustc_codegen_cranelift.dylib" \
cargo +nightly-2026-01-02 build \
  --target aarch64-unknown-redox-clif.json \
  --release \
  -Z build-std=core,alloc \
  -Zbuild-std-features=compiler_builtins/no-f16-f128
```

**Result:**
```
librelibc.a: 16.6 MB
ELF 64-bit LSB relocatable, ARM aarch64
```

### aarch64 QEMU Boot Status

The existing aarch64 Redox ISO boots successfully in QEMU:
```bash
qemu-system-aarch64 -M virt -cpu cortex-a72 -m 2G \
  -device ramfb -device qemu-xhci -device usb-kbd -device usb-tablet \
  -bios /opt/homebrew/opt/qemu/share/qemu/edk2-aarch64-code.fd \
  -drive file=build/aarch64/desktop/redox-live.iso,format=raw,if=virtio \
  -nographic
```

**Boot Log:**
```
Redox OS Bootloader 1.0.0 on aarch64/UEFI
RedoxFS 67bdb861-27ba-47d9-9c44-8bb69b5386da: 647 MiB
kernel: 1/1 MiB
initfs: 24/24 MiB
kernel_entry(...)
kernel::arch::aarch64::device::serial:INFO -- serial_port virq = 33
```

**Blocker for Cranelift kernel boot:**
- Replacing the kernel in the RedoxFS image requires macFUSE kernel extension (needs reboot to enable)
- Or building a fresh image with `redox_installer`

**Cranelift aarch64 kernel properties:**
```
Entry point: 0xFFFFFF000006CE50
.text: 1.77 MB at 0xFFFFFF0000001000
.rodata: 304 KB
.data + .got: ~370 KB
Machine: AArch64
```

**Current Status:**
| Architecture | Kernel | relibc | QEMU Boot |
|--------------|--------|--------|-----------|
| x86_64 | ✅ | ✅ | ✅ |
| aarch64 | ✅ | ✅ | 🔧 (needs image rebuild) |

### Next Steps for aarch64 Boot
1. Enable macFUSE kernel extension (requires reboot)
2. Or use Linux VM to mount and modify RedoxFS
3. Or build fresh image with `make ARCH=aarch64` using Cranelift toolchain
