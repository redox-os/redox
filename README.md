Fork of Redox OS - Pure Rust Build

<h2 align="center">100% Rust — No LLVM Required</h2>

<b>Redox OS can now be compiled using a pure Rust toolchain.</b><br>
The kernel boots and relibc compiles using <a href="https://github.com/rust-lang/rustc_codegen_cranelift">Cranelift</a> — no C++ dependencies.
</p>

## Status: aarch64 ls WORKS!

Using virtio-9p for direct access to host filesystem on mac!

### What works:
- **server-cranelift.iso** - boots, `ls /` works, 9p host share works!
- `simple-ls` - pure Rust ls compiled with Cranelift
- `virtio-9pd` - 9p filesystem driver for host sharing
- Full boot to login prompt

### What doesn't work:
- `run-cranelift-redox-x86.sh` works via cranelift-redox-x86-ok.img but is NOT what we want (aarch64)
- `run-cranelift-aarch64-official.sh` with official iso does NOT work
- DO NOT USE `build/aarch64/server-official.iso` - known to NOT WORK
- uutils `ls` has localization bug - use `simple-ls` instead

The new build-cranelift.sh uses:
- Cranelift - codegen backend (no LLVM)
- rust-lld - linker (no GCC)
- llvm-ar/strip - from Rust toolchain
- libm crate - contrib/pure-rust/math_libm.rs replaces openlibm

Usage

    # Default aarch64 builds
  ./build-cranelift.sh kernel     # Build kernel
  ./build-cranelift.sh relibc     # Build relibc
  ./build-cranelift.sh drivers    # Build drivers
  ./build-cranelift.sh all        # Full build
  ./build-cranelift.sh shell      # Enter build shell
  ./build-cranelift.sh env        # Show configuration

  # Any of these work for x86_64:
  ARCH_x86=1 ./build-cranelift.sh kernel
  ARCH=x86_64 ./build-cranelift.sh kernel
  ./build.sh -X qemu



⏺ To rebuild and test:

  # 1. Build initfs binaries (ls, drivers, etc.)
  cd /opt/other/redox/recipes/core/base/source
  ./build-initfs-cranelift.sh

  # 2. Inject new initfs into ISO
  cd /opt/other/redox
  ./build/fstools/bin/redoxfs build/aarch64/server-cranelift.iso /tmp/mnt
  cp /tmp/initfs-cranelift.img /tmp/mnt/boot/initfs
  umount /tmp/mnt

  # 3. Run with 9p share
  ./run-9p.sh

  Or the quick way if binaries are already built:

  cd /opt/other/redox
  ./run-9p.sh

  The ISO at build/aarch64/server-cranelift.iso already has the updated initfs with ls working.