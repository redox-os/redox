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

### aarch64 QEMU Boot - SUCCESS! 🎉

On 2026-01-05, the Cranelift-compiled aarch64 kernel booted in QEMU!

**How to replace kernel (requires macFUSE):**
```bash
# Mount RedoxFS image
mkdir -p /tmp/redoxfs-mount
/opt/other/redox/build/fstools/bin/redoxfs build/aarch64/desktop/redox-live.iso /tmp/redoxfs-mount

# Replace kernel with Cranelift version
llvm-strip -o /tmp/kernel-stripped recipes/core/kernel/source/target/aarch64-redox-none/release/kernel
cp /tmp/kernel-stripped /tmp/redoxfs-mount/boot/kernel

# Unmount
umount /tmp/redoxfs-mount
```

**QEMU Command:**
```bash
qemu-system-aarch64 -M virt -cpu cortex-a72 -m 2G \
  -device ramfb -device qemu-xhci -device usb-kbd -device usb-tablet \
  -bios /opt/homebrew/opt/qemu/share/qemu/edk2-aarch64-code.fd \
  -drive file=build/aarch64/desktop/redox-live.iso,format=raw,if=virtio \
  -serial file:/tmp/qemu-serial.log \
  -monitor unix:/tmp/qemu-monitor.sock,server,nowait \
  -display none
```

**Boot Log (Cranelift kernel):**
```
Redox OS Bootloader 1.0.0 on aarch64/UEFI
RedoxFS 67bdb861-27ba-47d9-9c44-8bb69b5386da: 647 MiB
USB HID driver spawned with scheme `usb.pci-00-00-02.0_xhci`
Finished graphical debug
########## Redox OS ##########
# Login with the following:  #
# `user`                     #
# `root`:`password`          #
##############################
redox login:
```

**Fixed Issue:** virtio-netd driver panic "not implemented: virtio_core: aarch64 enable_msix"
- **Fix:** Implemented legacy INTx# interrupt fallback in `virtio-core/src/arch/aarch64.rs`
- **Commit:** 450df8b in recipes/core/base/source (local clone)
- MSI-X on aarch64 requires GICv3 ITS which isn't fully implemented
- Falls back to legacy pin-based interrupts when available

**Build Scripts Created:**
- `build-virtio-netd-aarch64.sh` - Builds virtio-netd with Cranelift relibc
- `run-cranelift-aarch64.sh` - Runs QEMU with optional kernel injection

**Current Limitation:**
Static binaries built with Cranelift relibc have ABI mismatch with ISO's dynamic runtime.
Full userspace rebuild needed for compatible drivers.

**Final Status:**
| Architecture | Kernel | relibc | QEMU Boot |
|--------------|--------|--------|-----------|
| x86_64 | ✅ | ✅ | ✅ |
| aarch64 | ✅ | ✅ | ✅ |

### Full Userspace Build with Cranelift - SUCCESS! 🎉

On 2026-01-05, built 46+ userspace binaries with Cranelift for aarch64!

**Build Requirements:**
1. Extract CRT objects from relibc archives:
   ```bash
   cd /tmp
   ar x /opt/other/redox/recipes/core/relibc/source/target/aarch64-unknown-redox-clif/release/libcrt0.a
   ar x .../libcrti.a
   ar x .../libcrtn.a
   # Copy crt0.o, crti.o, crtn.o to relibc release dir
   ```

2. Create unwind stubs for aarch64:
   ```c
   // unwind_stubs.c - compile with: clang -target aarch64-unknown-linux-gnu -c
   _Unwind_Word _Unwind_GetGR(...) { return 0; }
   // ... other stubs
   ```

3. Build command:
   ```bash
   RELIBC_DIR=/opt/other/redox/recipes/core/relibc/source/target/aarch64-unknown-redox-clif/release
   RUSTFLAGS="-Zcodegen-backend=.../librustc_codegen_cranelift.dylib \
     -L $RELIBC_DIR -Cpanic=abort \
     -Clink-arg=-z -Clink-arg=muldefs \
     -Clink-arg=-lunwind_stubs \
     -Clink-arg=$RELIBC_DIR/crt0.o \
     -Clink-arg=$RELIBC_DIR/crt0_rust.o \
     -Clink-arg=$RELIBC_DIR/crti.o \
     -Clink-arg=$RELIBC_DIR/crtn.o"
   cargo +nightly-2026-01-02 build --workspace \
     --target aarch64-unknown-redox-clif.json \
     --release -Z build-std=std,core,alloc,panic_abort
   ```

**Built Binaries (46 total):**
- System daemons: init, logd, randd, zerod, ipcd, ptyd, ramfs, audiod
- Network: smolnetd, dhcpd (dependencies)
- PCI: pcid, pcid-spawner, acpid, hwd
- Storage: ahcid, nvmed, ided, virtio-blkd, bcm2835-sdhcid, lived, usbscsid
- Graphics: vesad, fbcond, fbbootlogd, bgad, ihdgd, virtio-gpud
- Network drivers: e1000d, rtl8139d, rtl8168d, alxd, ixgbed, virtio-netd
- USB: xhcid, usbhidd, usbhubd, usbctl
- Audio: ac97d, ihdad, sb16d
- Other: redoxfs, inputd, redoxerd, vboxd, rtcd

**QEMU Test Results (aarch64):**
```
pcid: PCI SG-BS:DV.F VEND:DEVI CL.SC.IN.RV
pcid: PCI 00-00:00.0 1B36:0008 06.00.00.00 6
pcid: PCI 00-00:01.0 1AF4:1000 02.00.00.00 2  (virtio-net)
pcid: PCI 00-00:02.0 1B36:000D 0C.03.30.01 12 XHCI
pcid: PCI 00-00:03.0 1AF4:1001 01.00.00.00 1  (virtio-blk)
smolnetd: no network adapter found
audiod: No such device
```

**Working Cranelift binaries:**
- ✅ pcid - enumerates PCI devices correctly
- ✅ smolnetd - runs, reports no network adapter
- ✅ audiod - runs, reports no audio device
- ❌ ipcd - crashes (needs investigation)
- ❌ pcid-spawner - fails to spawn drivers

**Key Fix:** Entry point was 0x0 without explicit CRT objects. Must link crt0.o, crti.o, crtn.o explicitly.

### Summary: Pure Rust Toolchain for Redox OS

Both x86_64 and aarch64 Redox kernels can now be compiled with Cranelift (pure Rust) instead of LLVM!

Full userspace (46+ binaries) now builds with Cranelift for aarch64. Some binaries run successfully in QEMU.

### ipcd Investigation - 2026-01-06

**Root Cause Found:** ipcd crashes during `fork()` because `has_proc_fd` is false in relibc.

**Why fork() fails:**
```rust
// In redox-rt/src/proc.rs:
assert!(
    proc_info.has_proc_fd,
    "cannot use ForkArgs::Managed without an existing proc info"
);
```

The `has_proc_fd` flag is set at compile time via `cfg!(feature = "proc")` in redox-rt.
Even though `proc` is a default feature, binaries compiled for the Cranelift target don't have it enabled properly.

**Attempted Workaround:**
Modified `daemon/src/lib.rs` to skip fork() entirely:
```rust
pub fn new<F: FnOnce(Daemon) -> !>(f: F) -> ! {
    let (_, write_pipe) = std::io::pipe().unwrap();
    f(Daemon { write_pipe })  // Run directly, no fork
}
```

**New Problem: CRT Initialization Crash**
When rebuilding ipcd with the no-fork daemon:
- Binary has correct entry point (0x35AE70)
- Crashes in `relibc_start_v1` during CRT initialization
- Error: UNHANDLED EXCEPTION at entry+0x1C
- FAR_EL1 shows memory access at 0x3A8378

**Required CRT Object Fix:**
CRT objects must include BOTH the `.o` and `.asm.o` files:
```bash
# crt0.o needs _start from asm.o
ld.lld -r -o crt0.o crt0-*.rcgu.o crt0-*.asm.o

# crti.o needs _init/_fini from asm.o
ld.lld -r -o crti.o crti-*.rcgu.o crti-*.asm.o
```

**Unresolved Issue:**
Even with proper entry point, the binary crashes during CRT initialization.
Likely cause: ABI mismatch between Cranelift-compiled code and LLVM-compiled relibc components in the ISO.

**Next Steps:**
1. Rebuild entire ISO with Cranelift-compiled userspace (not just individual binaries)
2. Or investigate the specific CRT initialization incompatibility
3. Consider using the official Redox cross-compiler for a clean test

**Build Script for ipcd:**
```bash
# Located at /opt/other/redox/build-ipcd.sh
RELIBC_DIR=/opt/other/redox/recipes/core/relibc/source/target/aarch64-unknown-redox-clif/release
RUSTFLAGS="-Zcodegen-backend=...librustc_codegen_cranelift.dylib \
  -L ${RELIBC_DIR} -Cpanic=abort \
  -Clink-arg=${RELIBC_DIR}/crt0.o \
  -Clink-arg=${RELIBC_DIR}/crt0_rust.o \
  -Clink-arg=${RELIBC_DIR}/crti.o \
  -Clink-arg=${RELIBC_DIR}/crtn.o"
```

**relibc Dependency Fix:**
To build relibc, both relibc and redox-rt must use the same syscall version:
```toml
# In Cargo.toml and redox-rt/Cargo.toml:
redox_syscall = "0.6.0"  # Not 0.7.0
redox_event = "=0.4.2"   # Pin exact version
# In redox-ioctl/Cargo.toml:
redox_syscall = "0.6.0"
```

### Full ISO Rebuild with Cranelift - 2026-01-06

**Build Script:** `build-cranelift-iso.sh`

A comprehensive build script was created to rebuild the entire ISO with Cranelift-compiled userspace.

**Components Built:**
| Component | Size | Status |
|-----------|------|--------|
| Kernel (aarch64) | 8.3 MB | ✅ |
| relibc | 16 MB | ✅ |
| Userspace binaries | 46+ | ✅ |

**Build Command:**
```bash
./build-cranelift-iso.sh aarch64 server all
```

**What it does:**
1. Creates custom target specs for Cranelift (max-atomic-width: 64)
2. Builds relibc with Cranelift backend
3. Sets up sysroot with CRT objects and unwind stubs
4. Builds kernel with Cranelift
5. Builds 46+ userspace drivers/daemons
6. Injects Cranelift binaries into existing ISO

**Userspace Binaries Built:**
- Core services: init, logd, randd, zerod, audiod, ipcd, ptyd
- Network: smolnetd
- PCI/System: pcid, pcid-spawner, acpid, hwd
- Storage: ahcid, nvmed, virtio-blkd, ided, lived, ramfs, usbscsid
- Graphics: vesad, fbcond, fbbootlogd, virtio-gpud, ihdgd
- USB: xhcid, usbhidd, usbhubd, usbctl
- Network drivers: e1000d, rtl8139d, rtl8168d, virtio-netd, alxd, ixgbed
- Other: inputd, redoxerd

**Target Specs:**
Custom target JSONs with `max-atomic-width: 64` (Cranelift doesn't support 128-bit atomics on aarch64):
- `tools/aarch64-redox-none.json` - Kernel target
- `tools/aarch64-unknown-redox-clif.json` - Userspace target

**Sysroot Structure:**
```
build/aarch64/cranelift-sysroot/
├── lib/
│   ├── libc.a (from relibc)
│   ├── crt0.o, crti.o, crtn.o
│   ├── libunwind_stubs.a
│   └── libpthread.a, libdl.a, librt.a (empty stubs)
└── include/
```

**QEMU Testing Notes:**
UEFI firmware (`edk2-aarch64-code.fd`) not available in Homebrew QEMU 10.2.0.
Download separately from https://github.com/tianocore/edk2 or use `brew install qemu-efi-aarch64`.
