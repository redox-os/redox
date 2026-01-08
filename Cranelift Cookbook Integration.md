  Cranelift Cookbook Integration

  Changes Made

  1. src/cook/script.rs:
    - Added COOKBOOK_CRANELIFT environment variable support
    - When enabled, adds -Z build-std=core,alloc and -Zbuild-std-features flags
    - Fixed RUSTFLAGS handling to preserve Cranelift backend flag when DYNAMIC_INIT sets flags
    - Fixed configure flags for native host builds (don't set --host on macOS)
  2. src/cook/cook_build.rs:
    - Added passthrough of COOKBOOK_CRANELIFT environment variable to build scripts
  3. build-cranelift.sh:
    - Updated to use cookbook integration
    - Sets up proper environment (DYLD_LIBRARY_PATH, RUSTFLAGS, RUSTUP_TOOLCHAIN)
    - Defaults to x86_64 (aarch64 has 128-bit atomics limitation in Cranelift)

  Verified Working

  - Kernel: Builds successfully with Cranelift for x86_64-unknown-none
  - relibc: Builds successfully with Cranelift for x86_64-unknown-redox

  Known Limitations

  1. aarch64: Cranelift doesn't support 128-bit atomics yet for aarch64 kernel targets
  2. macOS host builds: The host:libtool build has issues with CC being set incorrectly by redoxer (the external crate). Using PREFIX_BINARY=1 for prebuilt toolchains is recommended on Linux.

  Usage

  cd /opt/other/redox
  ./build-cranelift.sh relibc   # Build relibc
  ./build-cranelift.sh kernel   # Build kernel  
  ./build-cranelift.sh shell    # Enter shell with Cranelift env

  The macOS host toolchain issue requires further investigation into how redoxer sets CC for host builds - the cookbook changes are in place, but redoxer (external crate) overrides CC even after our fixes.