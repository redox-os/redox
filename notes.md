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


# Legacy x86_64 (use ARCH_x86=1)
ARCH_x86=1 ./build-cranelift.sh kernel
ARCH=x86_64 ./build-cranelift.sh kernel
./build.sh -X qemu

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

### virtio-9p Host Filesystem Sharing - SUCCESS! 🎉

### macOS NBD Limitation (qemu-nbd)

macOS does not provide a kernel NBD driver, so `qemu-nbd` does not work natively.
Use the qcow2 -> raw conversion + `redoxfs` workflow instead, or run `qemu-nbd`
inside a Linux VM if a true NBD device is required.

On 2026-01-06, successfully implemented virtio-9p filesystem sharing between QEMU host and Redox guest!

**The Problem:**
Initial implementation hung on the second virtio queue transaction because `futures::executor::block_on()`
doesn't work with virtio's async completion mechanism without a proper event loop to handle interrupts.

**The Fix:**
Replaced `block_on()` with a simple spin-polling function that repeatedly polls the future until ready:
```rust
fn spin_poll<F: std::future::Future>(mut future: F) -> F::Output {
    // Create no-op waker and spin until ready
    loop {
        match future.poll(&mut cx) {
            Poll::Ready(result) => return result,
            Poll::Pending => core::hint::spin_loop(),
        }
    }
}
```

**Test Results:**
```
test-9p: opening /scheme/9p.hostshare/test.txt
test-9p: read 42 bytes: Hello from host filesystem via virtio-9p!
test-9p: SUCCESS!
```

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

This enables rapid testing without rebuilding the ISO - just modify files on the host!

### Cranelift Userspace Binary Execution - SUCCESS! - 2026-01-07

Successfully executed a Cranelift-compiled userspace binary (`simple-ls`) on Redox aarch64!

**Build Command:**
```bash
cd recipes/core/base/source
./build-simple-ls.sh  # Builds and places ls at /tmp/9p-share/ls
```

**Boot Log showing ls execution:**
```
init: running: ls /scheme/
event  memory  pipe  sys  time  kernel.dtb  kernel.acpi  debug  irq  kernel.proc  serio  initfs  proc  null  zero  rand  log  logging  input  fbbootlog  disk.live  acpi  pci  disk.pci-00-00-02.0_virtio_blk  9p.hostshare
```

**Key Fix - 9p Read Buffer:**
The virtio-9p driver's read count must be limited to fit the response in msize:
```rust
// In client.rs read():
let max_data = self.msize.saturating_sub(7 + 4);  // header + data_len field
let count = count.min(max_data);
```

**simple-ls location:** `recipes/core/base/source/simple-ls/`

This completes the proof that the entire Rust toolchain (Cranelift codegen) can produce working Redox userspace binaries!

### Raw QEMU Boot Helper

Added `run-dev-raw.sh` to boot directly from a raw disk image (no ISO backing).
Defaults to `/tmp/dev-raw.img` or override with `RAW_IMG=...`.

### Raw Base Image Overlay (pure-rust.img)

Converted `build/aarch64/pure-rust.iso` to raw:
`qemu-img convert -O raw build/aarch64/pure-rust.iso build/aarch64/pure-rust.img`

Created overlay (note: use absolute backing path to avoid path duplication issues):
`qemu-img create -f qcow2 -b /opt/other/redox/build/aarch64/pure-rust.img -F raw build/aarch64/dev.qcow2`




⏺ You need to configure Ion inside Redox to use the 9p path. Run these commands in your running QEMU session:

  # Create config dir and copy the initrc
  mkdir -p ~/.config/ion
  cp /scheme/9p.hostshare/ion-initrc ~/.config/ion/initrc

  # Verify it's there
  cat ~/.config/ion/initrc

  # Source it now (or restart shell)
  source ~/.config/ion/initrc

  # Test - this should write to 9p
  echo "test command"
  cat /scheme/9p.hostshare/ion_history

  If Ion doesn't respect ~/.config/ion/initrc, try the alternative location:
  mkdir -p ~/.ion
  cp /scheme/9p.hostshare/ion-initrc ~/.ion/initrc

  Or set it directly for this session:
  let HISTFILE = "/scheme/9p.hostshare/ion_history"

  Then type some commands and check if /tmp/9p-share/ion_history updates on your Mac.



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
## 2026-01-08: 9p driver and coreutils investigation

### 9p O_DIRECTORY fix (DONE)
- Commit 426d3458e added patch to remove ENOTDIR check blocking cat on 9p files
- The patch in `patches/virtio-9pd-o_directory.patch` was already applied to scheme.rs
- Test-9p now shows `O_DIRECTORY TEST SUCCESS!`
- Files can be opened via 9p without ENOTDIR error

### Coreutils issue (NOT FIXED)
- Issue predates this session - even `pure-rust.WORKS.iso` has it
- All coreutils commands (cat, rm, mkdir) show `ls:` prefix errors
- The coreutils binary appears to be corrupted/replaced with simple-ls behavior

### Why coreutils build fails:
1. C dependencies (blake3, onig) fail cross-compilation
2. Missing relibc std functions (StdoutLock methods, rayon threading)
3. Requires full Redox cookbook build environment

### Working coreutils:
- The `pure-rust.WORKS.iso` coreutils is 2MB, statically linked
- Official coreutils is 11MB, dynamically linked
- Both have the same `ls:` prefix bug

### Build script created:
- `build-coreutils.sh` - attempts to build uutils with Cranelift
- Fails due to relibc missing std functions

### Next steps for coreutils fix:
1. Use full cookbook build system with proper cross-compilation
2. Or create minimal Rust implementations of essential commands
3. Or fix relibc to implement missing std functions

## 2026-01-08: Simple-coreutils Success

### Problem
- uutils coreutils was broken (all commands showed "ls:" prefix due to corrupted multicall binary)
- Building uutils with Cranelift failed due to C dependencies (blake3, onig)
- Official coreutils had localization errors

### Solution
Created simple-coreutils package with standalone implementations:
- simple-cat, simple-cp, simple-rm, simple-mkdir, simple-echo, simple-touch
- Built with Cranelift for aarch64
- Injected into ISO at /usr/bin/
- Symlinks created: cat -> simple-cat, etc.

### Key Files
- `/recipes/core/base/source/simple-coreutils/` - source code
- `/recipes/core/base/source/build-simple-coreutils.sh` - build script
- Binaries at `/tmp/simple-coreutils/` after build

### Test Results
```
init: running: rm -rf /tmp       # WORKS (no error)
init: running: mkdir -m a=rwxt /tmp  # WORKS (no error)
simple-cat /scheme/9p.hostshare/hello  # shows file content
ls /scheme/9p.hostshare/  # lists files correctly
```

### QEMU 10.2.0 Note
The error "drive with bus=0, unit=0 exists" appears when running QEMU commands 
directly but bash -c wrapper works around it. Use run-backup.sh pattern for reliable boots.

## 9p Write Fix Investigation (Thu Jan  8 17:04:09 CET 2026)

Fixed two bugs in virtio-9pd/src/scheme.rs:

1. **lcreate already opens the file**: After calling lcreate(), the fid is already
   opened for I/O. The code was calling lopen() again which would fail with a 9P
   protocol error.

2. **O_CREAT passed to lopen()**: The to_9p_flags() function was passing O_CREAT
   to lopen(), but lopen() doesn't create files - that's what lcreate() is for.
   Added new to_9p_lopen_flags() function that excludes O_CREAT.

Changes made:
- Added to_9p_lopen_flags() method to convert Redox flags to 9P lopen flags
- Track whether lcreate was used with 'already_opened' flag  
- Skip lopen() call if file was already opened by lcreate()

Committed to recipes/core/base/source submodule.

Testing blocked: Initfs rebuild produces different image that crashes.
Need to investigate proper initfs rebuild process.

Added run-utm.sh to boot with UTM-bundled QEMU and virtio-9p share support.

## Symlink feature failed (2026-01-08)
- Cannot add 'ln -s /scheme/9p.hostshare /root/host' to init.rc
- Reason: cranelift-initfs/initfs/bin binaries are broken, rebuilding initfs crashes boot
- redox-initfs-ar requires bootstrap + initfs dir, no way to extract/modify working initfs
- Workaround: run 'ln -s /scheme/9p.hostshare /root/host' manually after login

## Raw img persistence (2026-01-08)
- Writing to /root/ok inside QEMU persisted during session, but after QEMU terminated the host-mounted raw img still had old contents (cat showed 123).
- Suspect unclean shutdown or different RAW_IMG path; need clean shutdown/sync to flush or verify RAW_IMG used by run-dev-img.sh.

- No `sync` binary found in `redox-mount/usr/bin`; added `simple-sync` to `simple-coreutils` for a lightweight sync helper.

- `cargo +nightly build` for `simple-sync` failed under sccache (Operation not permitted); succeeded with `RUSTC_WRAPPER=` and `-Zbuild-std` for `aarch64-unknown-redox-clif`.
- Built binary copied to `share/simple-sync` for 9p usage.


## Ion Shell: "." (dot) builtin not registered

**Issue**: `. .ionrc` fails with "Exec format error (os error 8)" while `source .ionrc` works.

**Root cause**: In Ion shell source (`src/lib/builtins/mod.rs`), the `source` command is registered as a builtin but `.` (dot) is NOT:
```rust
.add("source", &builtin_source, SOURCE_DESC)
// MISSING: .add(".", &builtin_source, SOURCE_DESC)
```

When user types `. .ionrc`, Ion doesn't recognize `.` as a builtin and tries to execute it as a command (hence ENOEXEC).

**Fix**: Add `.` as an alias for `source` in Ion's `with_basic()` function:
```rust
.add(".", &builtin_source, SOURCE_DESC)
```

**Blocked on**: Rebuilding Ion for Redox requires cross-compiling the `calculate` crate which has C dependencies (`decimal`).

**Workaround**: Use `source` instead of `.` in Redox scripts until upstream is patched.

**Upstream**: https://gitlab.redox-os.org/redox-os/ion

## Ion alias syntax note
- Function names must be alphanumeric only (no '.')
- Tried: fn . file ... end => rejected
- Try: alias . source (may or may not work depending on Ion version)
- Confirmed workaround: just use 'source script' instead of '. script'


## 2026-01-08 fbcond boot error fix
Fixed GUARD PAGE crash in fbcond during boot. The issue was unwrap() calls in 
display.rs reopen_for_handoff() panicking when display wasn't ready. Now handles
errors gracefully and logs warning instead. Committed in drivers submodule as 25241ec0.


## 2026-01-09 - initfs vs redoxfs mount clarification
- /scheme/initfs/bin/* is embedded in boot image, NOT in redox-mount-works
- To recover initfs crashes: restore entire .img from working backup
- redox-mount-works only contains the main filesystem (/usr, /home, /etc)


## 2026-01-09 - PIE fix SUCCESS!
- Fixed 'position-independent-executables': true -> false in all Cranelift target specs
- Rebuilt ALL initfs binaries with PIE:false
- Boot now reaches login prompt with Cranelift-compiled initfs
- Files fixed: tools/*.json, recipes/core/*/source/*.json, build-cranelift.sh
- Snapshot: pure-rust.works.img


## 2026-01-09: initrc build version display
Added build version/commit/date to Ion shell initrc:
- File: ~/.config/ion/initrc 
- Shows: 'Pure-Rust Redox | Build: <commit> | <date>'
- Applied to both pure-rust.img and redox-mount-works

                                                                   
###
     kernel::arch::aarch64::interrupt::exception:ERROR -- FATAL: Not an SVC induced synchronous exception (ty=111100)
     FAR_EL1: 0xe1a8
happens when ... ?
## 2026-01-09 virtio-netd crash fix
Fixed UNHANDLED EXCEPTION in virtio-netd using same pattern as fbcond fix.
Replaced unwrap()/expect()/assert!/unimplemented!() with graceful error handling.
Changes in recipes/core/base/source/drivers/net/virtio-netd/
Commit: 0db15443 in base/source submodule
NOTE: Source-only fix - cannot rebuild drivers (initfs binaries are broken per CLAUDE.md)


## init.rc build version locations
- recipes/core/base/source/init.rc (source)
- build/aarch64/pure-rust-initfs/etc/init.rc
- build/aarch64/cranelift-initfs/initfs/etc/init.rc
Format: # Pure-Rust Redox | Build: <commit> | <date>


## 2026-01-09 virtio-netd crash analysis & fix

### Root Cause
1. LLVM-built binary (in image) panics at:
   `drivers/virtio-core/src/arch/aarch64.rs:8 - not implemented: virtio_core: aarch64 enable_msix`
   
2. Source has fix (uses legacy INTx fallback) but can't rebuild because:
   - Cranelift binaries have entry point 0x0 (ELF loading broken)
   - llvm-objdump shows: `start address: 0x0000000000000000`

### Fix Applied
- Removed /etc/pcid.d/virtio-netd.toml from image
- Driver no longer spawned at boot
- System boots cleanly to login prompt

### Source Code Fixes (for future rebuilds)
- virtio-netd/src/main.rs: graceful error handling 
- virtio-netd/src/scheme.rs: VirtioNet::new returns Result
- virtio-core/arch/aarch64.rs: already has MSI-X fallback (source is fixed)

Commits:
- 6a06c53a (base/source): virtio-netd graceful error handling
- 8098827e3 (main): disabled virtio-netd pcid config

### Future Work
To enable networking, need to either:
1. Fix Cranelift ELF entry point issue
 - virtio-netd/src/main.rs: graceful error handling 
- virtio-netd/src/scheme.rs: VirtioNet::new returns Result
2. Or cross-compile with LLVM and inject updated binaries NO! NO LLVM!

Fixed GUARD PAGE crash in fbcond during boot. The issue was unwrap() calls in
display.rs reopen_for_handoff() panicking when display wasn't ready. Now handles
errors gracefully and logs warning instead. Committed in drivers submodule as 25241ec0.


## Build Version Tracking
  CLAUDE.md note added listing all files to update on each build:
  Update these files with current commit/date on each significant build:
  - recipes/core/base/source/init.rc (source, line 1 comment)
  - build/aarch64/pure-rust-initfs/etc/init.rc (initfs)
  - build/aarch64/cranelift-initfs/initfs/etc/init.rc (cranelift initfs)
  - ~/.config/ion/initrc in mounted images (login message)
## 2026-01-09 Cranelift ELF entry point FIXED!

### Root Cause
- Target spec used `gnu-lld` linker-flavor directly
- Unlike `gnu-cc`, gnu-lld doesn't auto-include crt0.o
- Result: No `_start` symbol, entry point was 0x0

### Solution
Added to tools/aarch64-unknown-redox-clif.json:
```json
"pre-link-objects": {
    "static-nopic-exe": ["crt0.o", "crti.o"]
},
"post-link-objects": {
    "static-nopic-exe": ["crtn.o"]
}
```

### Result
- virtio-netd entry point: 0x323e50 (was 0x0)
- virtio-core MSI-X fallback works on aarch64
- Both queues (RX/TX) enabled successfully
- System boots cleanly with network driver loaded!

Commits:
- 9dfae9134 (main): target spec with CRT objects
- 8b003702 (base/source): copied target spec
