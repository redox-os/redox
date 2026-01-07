# Redox OS Cranelift Build - Current State

**Updated:** 2026-01-07

## Current Status: aarch64 Boots to Login Prompt ✅

The pure Rust aarch64 build boots fully and can execute commands.

```bash
./run-9p.sh  # Boots server-cranelift.iso with host filesystem sharing
```

## What Works

| Component | Status |
|-----------|--------|
| Kernel (Cranelift) | ✅ Boots |
| relibc (Cranelift) | ✅ Works |
| init, bootstrap | ✅ Run init.rc fully |
| All initfs daemons | ✅ Fork and run |
| virtio-blkd | ✅ Disk access works |
| virtio-9pd | ✅ Host filesystem sharing |
| simple-ls | ✅ Works (`ls /`) |
| Login prompt | ✅ Reached |

## What Doesn't Work

| Issue | Notes |
|-------|-------|
| fbcond | Crashes in nographic mode (expected) |
| virtio-netd | MSI-X not implemented for aarch64 |
| uutils ls | Localization bug - use `simple-ls` instead |
| Official ISO | `server-official.iso` does NOT boot |

## Architecture

Hybrid build using official ISO base with Cranelift initfs:

| Component | Source |
|-----------|--------|
| ISO base | Official Redox build |
| Kernel | Official (LLVM) or Cranelift |
| initfs | Cranelift-built binaries |
| rootfs | Official |

## Key Files

| File | Purpose |
|------|---------|
| `build/aarch64/server-cranelift.iso` | Working bootable ISO |
| `build-cranelift.sh` | Main build script |
| `run-9p.sh` | QEMU runner with 9p sharing |
| `recipes/core/base/source/build-initfs-cranelift.sh` | Build initfs binaries |

## Pure Rust Toolchain

- **Cranelift** - codegen backend (no LLVM)
- **rust-lld** - linker (no GCC)
- **llvm-ar/strip** - from Rust toolchain
- **libm crate** - replaces openlibm

## Next Steps

1. Build kernel with Cranelift for fully pure ISO
2. Fix virtio-netd MSI-X for networking
3. Build rootfs userspace with Cranelift
