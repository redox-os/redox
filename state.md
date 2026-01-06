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

### Bootstrap Built with Cranelift - SUCCESS! 🎉

**Commit:** `d13f8aa7` in `recipes/core/base/source`

Bootstrap now builds and links with Cranelift for aarch64!

**Build command:**
```bash
cd recipes/core/base/source/bootstrap
./build-cranelift.sh
```

**Output:**
```
bootstrap-cranelift-stripped: 820 KB
ELF 64-bit LSB executable, ARM aarch64, statically linked
Entry point: 0x3000
```

### Initfs Binaries Built with Cranelift - SUCCESS! 🎉

**Commit:** `527c979c` in `recipes/core/base/source`

All 18 initfs binaries now build with Cranelift:
- Core: init, logd, ramfs, randd, zerod, nulld
- Drivers: pcid, pcid-spawner, acpid, fbbootlogd, fbcond, hwd, inputd, lived, nvmed, rtcd, vesad
- Virtio: virtio-blkd, virtio-gpud
- Plus: redoxfs, bootstrap

**Build command:**
```bash
cd recipes/core/base/source
./build-initfs-cranelift.sh
```

**Output:**
```
initfs-cranelift.img: 49 MB
18 binaries @ ~2.2 MB each (stripped)
```

### Bootstrap Boots and Launches Init - SUCCESS!

**Commits:**
- `653dbb9d` in `recipes/core/base/source` - fix bootstrap scheme opens
- `b91aab5e` in `recipes/core/relibc/source` - fix FdGuard::open and make_init

**Problem:** `openat(0, path)` returns EOPNOTSUPP for scheme paths on Redox kernel.

**Fix:** Use legacy `SYS_OPEN` syscall (opcode 5) instead of `openat` for all scheme path opens:
- `bootstrap/src/lib.rs`: Added `compat::open_fd()` helper
- `bootstrap/src/exec.rs`: Use `compat::open_fd()` for `/scheme/kernel.proc/authority` and `/scheme/sys/env`
- `bootstrap/src/procmgr.rs`: Use `compat::open_fd()` for `/scheme/event`
- `relibc/redox-rt/src/proc.rs`: Added `legacy_open_for_fdguard()` for `FdGuard::open` and `make_init()`

**QEMU Boot Log (2026-01-06):**
```
kernel::syscall::process:DEBUG -- Bootstrap entry point: 3000
kernel::scheme::user:DEBUG -- call_fdread: payload: 8 metadata: 2
UNHANDLED EXCEPTION, CPU #0, PID 1, NAME /scheme/initfs/bin/init
```

Bootstrap now successfully:
1. Opens `/scheme/kernel.proc/authority`
2. Creates event queue (`/scheme/event`)
3. Creates init process (`/scheme/proc/init`)
4. Executes init binary from initfs

### Init Entry Point 0x0 Bug - FIXED!

**Root Cause:** Cranelift-built binaries were missing CRT objects (crt0.o, crti.o, crtn.o) which contain `_start`.

**Fix 1 - Add CRT objects to RUSTFLAGS:**
```bash
-Clink-arg=${RELIBC}/crt0.o \
-Clink-arg=${RELIBC}/crt0_rust.o \
-Clink-arg=${RELIBC}/crti.o \
-Clink-arg=${RELIBC}/crtn.o
```

**Fix 2 - Use static relocation model:**
PIE (Position Independent Executable) binaries have entry point 0x0 which requires dynamic relocation.
The kernel/bootstrap ELF loader doesn't handle PIE relocation.
Added `-Crelocation-model=static` to produce static executables.

**Before:**
```
Type: DYN (Shared object file)
Entry point: 0x0
```

**After:**
```
Type: EXEC (Executable file)
Entry point: 0x40FFEC
_start symbol present
```

### Current Status: Init Runs and Executes init.rc!

**QEMU Boot Log (2026-01-06):**
```
init: running: export PATH /scheme/initfs/bin
init: running: export RUST_BACKTRACE 1
init: running: rtcd
init: running: nulld
init: running: zerod
init: running: randd
init: running: logd
init: running: inputd
init: running: vesad
init: running: fbbootlogd
init: running: fbcond 2
init: running: lived
init: running: run /scheme/initfs/etc/init_drivers.rc
init: running: pcid-spawner /scheme/initfs/etc/pcid/initfs.toml
init: running: redoxfs --uuid $REDOXFS_UUID file $REDOXFS_BLOCK
init: running: cd /
init: running: export PATH /usr/bin
init: running: run.d /usr/lib/init.d /etc/init.d
init: running: ipcd
```

The Cranelift-compiled init binary:
1. ✅ Runs without crashing
2. ✅ Reads and parses init.rc
3. ✅ Forks daemons properly (nulld, zerod, randd, logd, etc.)
4. ✅ Starts graphics stack (vesad, fbbootlogd, fbcond)
5. ✅ Runs driver spawner (pcid-spawner)
6. ✅ Mounts redoxfs root filesystem
7. ✅ Transitions to /usr/bin PATH
8. ✅ Runs init.d scripts

### Daemon Fork Fix

**Problem:** Daemon crate was modified to skip forking as a workaround, causing init to block forever waiting for daemons.

**Fix:** Restored proper fork() in `daemon/src/lib.rs`:
```rust
match unsafe { libc::fork() } {
    0 => { /* child continues as daemon */ }
    _pid => { /* parent waits for ready signal, then exits */ }
}
```

### virtio-blkd IRQ Conflict - FIXED!

**Commit:** `de8c0066` in kernel source

**Problem:** All drivers failed with "failed to open IRQ file: File exists (os error 17)" when trying to register for interrupts.

**Root Cause:** During IRQ chip initialization (GIC, GICv3, BCM2835, BCM2836), all IRQ descriptors were marked as `used = true`. The IRQ scheme's `is_reserved()` function checked this field, returning EEXIST for any driver IRQ open.

**Fix:** Changed all IRQ chip init code to set `used = false`:
```rust
// Before (broken):
irq_desc[idx + i].basic.used = true;

// After (fixed):
irq_desc[idx + i].basic.used = false;  // Available for driver reservation
```

**Result:** virtio-blkd now successfully opens IRQ file and disk driver works:
```
kernel::scheme::irq:DEBUG -- open_phandle_irq virq=38
virtio-blk: disk size: 1331200 sectors and block size of 512 bytes
```

### Remaining Issues

1. **fbcond panic** - "No display present" in headless mode (expected)
2. **ipcd hang** - Init stuck waiting for ipcd to start (needs investigation)

### Next Steps

1. Investigate ipcd hang
2. Test with display (-display gtk)
3. Reach login prompt

## Files Modified

| File | Change |
|------|--------|
| `recipes/core/kernel/source/src/arch/aarch64/device/irqchip/gic.rs` | IRQ init: used = false |
| `recipes/core/kernel/source/src/arch/aarch64/device/irqchip/gicv3.rs` | IRQ init: used = false |
| `recipes/core/kernel/source/src/arch/aarch64/device/irqchip/irq_bcm2835.rs` | IRQ init: used = false |
| `recipes/core/kernel/source/src/arch/aarch64/device/irqchip/irq_bcm2836.rs` | IRQ init: used = false |
| `recipes/core/base/source/daemon/src/lib.rs` | Restored fork() for daemon daemonization |
| `recipes/core/base/source/drivers/pcid-spawner/src/main.rs` | Added retry loop |
| `recipes/core/base/source/.cargo/config.toml` | Relibc patches |
| `recipes/core/base/source/redox-scheme/*` | Local redox-scheme patch for syscall 0.7.0 |
| `recipes/core/base/source/Cargo.toml` | Added redox-scheme patch |
| `recipes/core/base/source/bootstrap/Cargo.toml` | Added redox-scheme patch |
| `recipes/core/base/source/bootstrap/src/lib.rs` | Added compat::open() and open_fd() |
| `recipes/core/base/source/bootstrap/src/start.rs` | Use compat::open() |
| `recipes/core/base/source/bootstrap/src/exec.rs` | Use compat::open_fd() for scheme paths |
| `recipes/core/base/source/bootstrap/src/initfs.rs` | Use compat::open() |
| `recipes/core/base/source/bootstrap/src/procmgr.rs` | Use compat::open_fd() for /scheme/event |
| `recipes/core/base/source/bootstrap/aarch64-unknown-redox-clif.json` | Custom target (64-bit atomics) |
| `recipes/core/base/source/bootstrap/build-cranelift.sh` | Build script |
| `recipes/core/base/source/build-initfs-cranelift.sh` | CRT objects + static relocation |
| `recipes/core/relibc/source/redox-rt/src/proc.rs` | Legacy SYS_OPEN for FdGuard::open/make_init |

## Build Scripts Created

- `/tmp/build-bootstrap.sh` - Attempts to build bootstrap with Cranelift
- `/tmp/build-redoxfs.sh` - Builds redoxfs with Cranelift
- `/opt/other/redox/build-cranelift.sh` - Main Cranelift build script
