Fork of Redox OS - Pure Rust Build

<h2 align="center">100% Rust — No LLVM Required</h2>

<b>Redox OS can now be compiled using a pure Rust toolchain.</b><br>
The kernel boots and relibc compiles using <a href="https://github.com/rust-lang/rustc_codegen_cranelift">Cranelift</a> — no C++ dependencies.
</p>

WIP: aarch64 

Using virtio-9p for direct access to host filesystem on mac!

run-cranelift-redox-x86.sh works via cranelift-redox-x86-ok.img but is NOT what we want (aarch64)
run-cranelift-aarch64-official.sh with official iso does NOT work! We need to build aarch64 from scratch:

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

# Legacy x86_64 (use ARCH_x86=1)
ARCH_x86=1 ./build-cranelift.sh kernel
ARCH=x86_64 ./build-cranelift.sh kernel
./build.sh -X qemu


