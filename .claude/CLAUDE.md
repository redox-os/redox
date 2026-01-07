# Redox OS Development Notes

## Cranelift Backend Experiment - SUCCESS!

### Architecture: aarch64 is the default!

All builds default to **aarch64** (ARM64). For legacy x86_64 builds, use `ARCH_x86=1`.

**Fork with fixes:** https://github.com/pannous/rustc_codegen_cranelift

### Key Changes Made

1. **Implemented `sym` operand support** (commit a7883ae1)
   - Added Mach-O underscore prefix for symbol names
   - Created wrapper functions for global_asm sym operands

2. **Fixed duplicate symbol errors** (commit 2106ade4)
   - Made wrapper names unique by including CGU name

3. **Fixed kernel Intel syntax** (commit ff9ac52c in kernel repo)
   - Changed `int $3` to `int 3` for Cranelift compatibility

### Pure Rust Toolchain

The build system now uses a pure Rust toolchain where possible:

| Component | Replacement |
|-----------|-------------|
| LLVM | Cranelift codegen backend |
| openlibm (C) | libm crate (Rust) via math_libm.rs |
| GNU ld/gcc | rust-lld linker |
| ar, strip | llvm-ar, llvm-strip from Rust |

### Working Build Commands

```bash
# Default: aarch64 builds
./build-cranelift.sh kernel     # Build aarch64 kernel
./build-cranelift.sh relibc     # Build aarch64 relibc
./build-cranelift.sh drivers    # Build aarch64 drivers
./build-cranelift.sh all        # Full aarch64 build
./build-cranelift.sh shell      # Enter Cranelift build shell

# Legacy x86_64 builds (use ARCH_x86=1)
ARCH_x86=1 ./build-cranelift.sh kernel   # x86_64 kernel
```

### Direct Builds (without cookbook)

```bash
cd recipes/core/kernel/source

# Use nightly-2026-01-02 to match Cranelift backend
DYLD_LIBRARY_PATH=/Users/me/.rustup/toolchains/nightly-2026-01-02-aarch64-apple-darwin/lib \
RUSTFLAGS="-Zcodegen-backend=/opt/other/rustc_codegen_cranelift/dist/lib/librustc_codegen_cranelift.dylib" \
cargo +nightly-2026-01-02 build \
  --target aarch64-unknown-none \
  --release \
  -Z build-std=core,alloc \
  -Zbuild-std-features=compiler-builtins-mem,compiler_builtins/no-f16-f128
```

### Build Cranelift Backend

```bash
cd /opt/other/rustc_codegen_cranelift
./y.sh prepare
./y.sh build --sysroot clif
```

### relibc Compiled with Cranelift - SUCCESS!

On 2026-01-04, relibc (Redox's C library) was compiled using Cranelift.

**Key Change:** Commit `a86211e4` added variadic function support!

### virtio-9p Host Filesystem Sharing - SUCCESS!

On 2026-01-06, successfully implemented virtio-9p filesystem sharing.

**QEMU Command with 9p Sharing:**
```bash
qemu-system-aarch64 -M virt -cpu cortex-a72 -m 2G \
    -bios tools/firmware/edk2-aarch64-code.fd \
    -drive file=build/aarch64/server-official.iso,format=raw,id=hd0,if=none \
    -device virtio-blk-pci,drive=hd0 \
    -device virtio-9p-pci,fsdev=host0,mount_tag=hostshare \
    -fsdev local,id=host0,path=/tmp/9p-share,security_model=none \
    -serial stdio
```

**Usage:**
1. Create files on host: `echo "test" > /tmp/9p-share/test.txt`
2. Access from Redox: `/scheme/9p.hostshare/test.txt`

### Cranelift Userspace Binary Execution - SUCCESS! - 2026-01-07

Successfully executed a Cranelift-compiled userspace binary (`simple-ls`) on Redox aarch64!

### Known Limitations

1. **aarch64 128-bit atomics**: Cranelift has max-atomic-width=64 for aarch64
2. **macOS host builds**: redoxer overrides CC; use PREFIX_BINARY=1

### Architecture Naming Convention

- **Default**: aarch64 (no suffix needed)
- **Legacy**: x86_64 (use `_x86` suffix or `ARCH_x86=1`)

Files and targets specific to x86_64 should be marked with `_x86` suffix.

### Pure Rust Math Library

The `contrib/pure-rust/math_libm.rs` file provides C-compatible exports wrapping the Rust `libm` crate, replacing openlibm (C).

To integrate into relibc:
```bash
cp contrib/pure-rust/math_libm.rs recipes/core/relibc/source/src/
# Add: mod math_libm; to lib.rs
```
