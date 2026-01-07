Fork of Redox OS - Pure Rust Build

<h2 align="center">100% Rust — No LLVM Required</h2>

<b>Redox OS can now be compiled using a pure Rust toolchain.</b><br>
The kernel boots and relibc compiles using <a href="https://github.com/rust-lang/rustc_codegen_cranelift">Cranelift</a> — no C++ dependencies.
</p>

WIP: aarch64 

New builds should be based on WORKING
build/aarch64/server-cranelift.iso
If it breaks restore from backup:
build/aarch64/server-cranelift.iso.bak
If it STILL breaks restore from last good backup:
build/aarch64/server-cranelift.iso.ok.bak

IGNORE build/aarch64/server-official.iso !!!

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
/opt/other/redox/build-cranelift.sh kernel     # Build kernel
.build-cranelift.sh relibc     # Build relibc
.build-cranelift.sh drivers    # Build drivers
.build-cranelift.sh all        # Full build
.build-cranelift.sh shell      # Enter build shell
.build-cranelift.sh env        # Show configuration

# Legacy x86_64 (use ARCH_x86=1)
ARCH_x86=1 ./build-cranelift.sh kernel
ARCH=x86_64 ./build-cranelift.sh kernel
./build.sh -X qemu


# FUSE is working on this Mac
redoxfs mount    | ✅ works

ALWAYS make a backup before modifying working iso disk images!

# TODOs

omit credentials while testing (disable redox login: prompt for now)

root:~# echo "write test $(date)" > /scheme/9p.hostshare/write-test.txt
ion: pipeline execution error: failed to redirect stdout to file '/scheme/9p.hostshare/write-test.txt': I/O error (os error 5)

See STATE.md for current state (may be out of sync, update often but carefully)


# FAQ

 The original initfs uses "RedoxFtw" magic, not "RedoxFS\0". 
 ./build/aarch64/cranelift-initfs/initfs-tools-target/release/redox-initfs-ar --output /tmp/pure-rust-initfs.img 

usually you want to cd into root dir
cd /opt/other/redox/

# QEMU Notes

HVF acceleration crashes aarch64 Redox! Use emulated CPU instead:
- ❌ `-accel hvf -cpu host` causes "Lacks grant" crashes in userspace
- ❌ `-accel hvf -cpu host` + `highmem=off` - flaky, sometimes works
- ❌ `-smp 4` with HVF - crashes (kernel only sees 1 CPU anyway)
- ✅ `-cpu cortex-a72` works reliably (slower but stable)
- run-backup.sh uses cortex-a72 - NEVER MODIFY IT (our fallback config!)
- Root cause: likely exception/memory handling in kernel under HVF

# OTHER

Don't push to gitlab upstream, just to the origin fork!

If you currently cannot boot / run a qemu session, just start a parallel one with different SOCK, similar to run-parallel.sh

If fixes work in the iso also apply them to build/aarch64/server-cranelift.qcow2 or use qcow2 directly, but create .bak !

