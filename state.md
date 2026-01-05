# Redox OS Cranelift Build - Current State

## Completed Work

### Kernel & relibc
- aarch64 kernel builds with Cranelift
- aarch64 relibc builds with Cranelift
- Both boot successfully in QEMU with original (LLVM) initfs

### Bug Fix: pcid-spawner Race Condition
**Commit:** `320f8cb0` in `recipes/core/base/source`

**Problem:** pcid-spawner fails with "No such device (os error 19)" because it tries to read `/scheme/pci` before pcid registers it.

**Fix:** Added retry loop in `drivers/pcid-spawner/src/main.rs`:
```rust
fn wait_for_scheme(path: &str, max_retries: u32, delay_ms: u64) -> Result<fs::ReadDir>
```
Waits up to 5 seconds (50 retries × 100ms) for the scheme to appear.

### Cranelift Initfs Build
- All 17 initfs binaries built with Cranelift for aarch64
- initfs archive created correctly (61MB stripped vs 26MB original)
- Config files (init.rc, init_drivers.rc, initfs.toml) included

## Current Blockers

### 1. Cranelift Initfs Boot Hang
Cranelift-compiled initfs hangs after kernel starts bootstrap. The LLVM-compiled bootstrap can't properly execute Cranelift-compiled binaries (possible ABI mismatch).

### 2. ~~Can't Build Bootstrap with Cranelift~~ FIXED!
**Commit:** `5b610770` in `recipes/core/base/source`

**Problem:** `redox-scheme` 0.8.3 uses `redox_syscall` 0.6.0, but bootstrap needs 0.7.0 for compatibility with `libredox` 0.1.12.

**Fix:**
- Created local `redox-scheme` patch updated for syscall 0.7.0
- Added manual handling for legacy opcodes (Open=0, Rmdir=1, Unlink=2) removed from 0.7.0
- Added `compat::open()` function in bootstrap for `SYS_OPEN` syscall removed from 0.7.0
- Patched workspace Cargo.toml to use local redox-scheme

Bootstrap now builds with Cranelift for aarch64!

### 3. No LLVM Toolchain
`aarch64-unknown-redox` is not a tier 2 Rust target, so no pre-built LLVM toolchain exists for cross-compilation on macOS.

## Next Steps

### Build Bootstrap with Cranelift
Now that redox-scheme compatibility is fixed, build bootstrap with Cranelift:
```bash
cd /opt/other/redox/recipes/core/base/source/bootstrap
DYLD_LIBRARY_PATH=~/.rustup/toolchains/nightly-2026-01-02-aarch64-apple-darwin/lib \
RUSTFLAGS="-Zcodegen-backend=/opt/other/rustc_codegen_cranelift/dist/lib/librustc_codegen_cranelift.dylib" \
cargo +nightly-2026-01-02 build \
  --target aarch64-unknown-redox \
  --release \
  -Z build-std=core,alloc \
  -Zbuild-std-features=compiler_builtins/no-f16-f128
```

### Create Full Cranelift Initfs
1. Build bootstrap with Cranelift
2. Build all initfs components with Cranelift
3. Create initfs archive
4. Test in QEMU

### Alternative: Test on x86_64
`x86_64-unknown-redox` is tier 2, so standard toolchain works. Could test pcid-spawner fix there first.

## Files Modified

| File | Change |
|------|--------|
| `recipes/core/base/source/drivers/pcid-spawner/src/main.rs` | Added retry loop |
| `recipes/core/base/source/.cargo/config.toml` | Relibc patches |
| `recipes/core/base/source/redox-scheme/*` | Local redox-scheme patch for syscall 0.7.0 |
| `recipes/core/base/source/Cargo.toml` | Added redox-scheme patch |
| `recipes/core/base/source/bootstrap/Cargo.toml` | Added redox-scheme patch |
| `recipes/core/base/source/bootstrap/src/lib.rs` | Added compat::open() |
| `recipes/core/base/source/bootstrap/src/start.rs` | Use compat::open() |
| `recipes/core/base/source/bootstrap/src/exec.rs` | Use compat::open() |
| `recipes/core/base/source/bootstrap/src/initfs.rs` | Use compat::open() |

## Build Scripts Created

- `/tmp/build-bootstrap.sh` - Attempts to build bootstrap with Cranelift
- `/tmp/build-redoxfs.sh` - Builds redoxfs with Cranelift
- `/opt/other/redox/build-cranelift.sh` - Main Cranelift build script
