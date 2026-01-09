Fork of Redox OS - Pure Rust Build

<h2 align="center">100% Rust — No LLVM Required</h2>

<b>Redox OS can now be compiled using a pure Rust toolchain.</b><br>
The kernel boots and relibc compiles using Cranelift — no C++ make or cmake dependencies.
</p>

We build ONLY for aarch64! 

New builds should are based on 
build/aarch64/pure-rust.iso via qcow2 !
If it breaks, restore by copying from the backup:
build/aarch64/pure-rust.WORKS.iso

Using virtio-9p ./share for direct access to host filesystem on mac!


# Development Workflow

## Quick Start

⚠️ Make a backup of our current image:
`cp build/aarch64/pure-rust.img build/aarch64/pure-rust.img.bak` before each session !

./run-dev.sh       # Unix socket Foreground or -s for /tmp/redox-dev-raw.sock

./old/run-debug.sh  # GDB TCP (telnet) daemonized ever needed?

## Injecting Files into Redox

• we want to use direct host file system integration ./share with 9P as often as possible. 

Until we have netd working GET MISSING pkg from 
https://static.redox-os.org/pkg/aarch64-unknown-redox/

NEW: 9P and/or qcow2
### Method 1: 9P Share (Runtime - Fastest)
Host files in /opt/other/redox/share/ appear at /scheme/9p.hostshare/ in Redox.
```bash
# On host:
cp my-tool /opt/other/redox/share/
# In Redox:
/scheme/9p.hostshare/my-tool
```
Good for: Testing binaries, scripts, quick iterations

### Method 2: Mount img
```bash
/opt/other/redox/mount-redox-mount.sh # same as:
# /opt/other/redox/build/fstools/bin/redoxfs /opt/other/redox/build/aarch64/pure-rust.img /opt/other/redox/redox-mount/
cp my-tool /opt/other/redox/redox-mount/usr/bin/ # or whatever
  # After editing, convert back:
umount /opt/other/redox/redox-mount/
```

IMPORTANT: 
ALWAYS test with /opt/other/redox/run-dev.sh after your injections!
If it works cp pure-rust.img with feature name, otherwise ask if we want to rollback or try again!

### Method 3: Rebuild initfs (For drivers/init)
```bash
# Edit files in recipes/core/base/source/
cd recipes/core/base/source && ./build-initfs-cranelift.sh
# Inject new initfs same as above
```

## Building Userspace Tools
```bash
# Build a tool with Cranelift for Redox
cd recipes/core/base/source
./build-initfs-cranelift.sh  # Builds all initfs tools
# Or build individual:
RUSTFLAGS="..." cargo +nightly build --target aarch64-unknown-redox-clif.json -p my-tool
```

# Cranelift
The new build-cranelift.sh uses:
- Cranelift - codegen backend (no LLVM)
- rust-lld - linker (no GCC)
- llvm-ar/strip - from Rust toolchain
- libm crate - contrib/pure-rust/math_libm.rs replaces openlibm

# Build Scripts
- `build-cranelift.sh` - Main Cranelift builder (kernel, relibc, drivers, all)
- `build-pure-rust-iso.sh` - Quick ISO assembly from pre-built binaries ONLY USE WHEN WE CAN'T patch via qcow2

## Usage
```bash
./build-cranelift.sh kernel   # Build kernel
./build-cranelift.sh relibc   # Build relibc
./build-cranelift.sh drivers  # Build drivers
./build-cranelift.sh all      # Full build
./build-cranelift.sh shell    # Enter build shell
```

# RECOVERY
pure-rust.works.img is always mounted at /opt/other/redox/redox-mount-works
copy it back to pure-rust.img if pure-rust.img is completely broken
copy selected files from redox-mount-works if only parts are broken

# Build Version Tracking
Update these files with current commit/date on each significant build:
- `recipes/core/base/source/init.rc` (source, line 1 comment)
- `build/aarch64/pure-rust-initfs/etc/init.rc` (initfs)
- `build/aarch64/cranelift-initfs/initfs/etc/init.rc` (cranelift initfs)
- `~/.config/ion/initrc` in mounted images (login message)

# TODOs

## 2026-01-09 virtio-netd boot crash fixed
LLVM binary panicked on "not implemented: virtio_core: aarch64 enable_msix".
Source has fix but Cranelift builds have entry point 0x0.
Workaround: Removed /etc/pcid.d/virtio-netd.toml from image.
Source fixes in 6a06c53a (base/source), see notes.md for details.

## 2026-01-08 fbcond boot error fix
Fixed GUARD PAGE crash in fbcond during boot. The issue was unwrap() calls in
display.rs reopen_for_handoff() panicking when display wasn't ready. Now handles
errors gracefully and logs warning instead. Committed in drivers submodule as 25241ec0.

⚠️ ATTENTION: cranelift-initfs/initfs/bin binaries are broken, rebuilding initfs crashes boot

 Risk: pre-built packages may not match Cranelift ABI

The target spec NEEDS "position-independent-executables": false:
The kernel's ELF loader doesn't support PIE relocation. Without this, binaries jump to address 0x0 on startup.
'true' would only be for security 🤷

# Get working redoxfs from mounted working image (it's inside initfs which we can't extract easily)


DID ANY modifications to redoxfs ever work??
init: running: rm -rf /tmp
kernel::context::memory:DEBUG -- Lacks grant
kernel::arch::aarch64::interrupt::exception:ERROR -- FATAL: Not an SVC induced synchronous exception (ty=100100)
FAR_EL1: 0x0
kernel::context::signal:INFO -- UNHANDLED EXCEPTION, CPU #0, PID 34, NAME /scheme/initfs/bin/redoxfs, CONTEXT 0xfffffe800012eea0
CRASH
 crash is in /scheme/initfs/bin/redoxfs which is embedded in the boot image, not the main filesystem. no easy fix

- **Config file**: `~/.config/ion/initrc` (not `.ionrc`!)

### Ion Shell "." (dot) command bug (IDENTIFIED)
**Root cause**: Ion doesn't register "." as a builtin alias for "source" in `src/lib/builtins/mod.rs`.
**Fix**: Add `.add(".", &builtin_source, SOURCE_DESC)` to `with_basic()` function.
**Workaround**: Use `source` instead of `.`

root:/scheme/9p.hostshare# cat hi
hi
root:/scheme/9p.hostshare# cat ba
cat: ba: I/O error (os error 5)

root:/scheme/9p.hostshare# df
Path            Size      Used      Free Use%
/scheme/memory   1422356    192756   1229600  13%
/scheme/logging   1422356    192756   1229600  13%
1970-01-01T00-05-26.968Z [@inputd:208 ERROR] invalid path ''

• coreutils broken - all commands show `ls:` prefix
  - 9p O_DIRECTORY fix applied (test-9p works)
  - But /usr/bin/coreutils binary is corrupted/replaced with simple-ls behavior
  - Need to rebuild uutils with Cranelift (blocked by relibc missing std functions)

• can't write to 9p share ( echo ok > ok works only in root )
root:~# echo "write test $(date)" > /scheme/9p.hostshare/write-test.txt
ion: pipeline execution error: failed to redirect stdout to file '/scheme/9p.hostshare/write-test.txt': I/O error (os error 5)

See STATE.md for current state (may be out of sync, update often but carefully)


# FAQ
⏺ Wrong tool! initfs needs "RedoxFtw" magic, not "RedoxFS\0". I used redoxfs-ar but should use redox-initfs-ar.

 ./build/aarch64/cranelift-initfs/initfs-tools-target/release/redox-initfs-ar --output /tmp/pure-rust-initfs.img 

usually you want to cd into root dir
cd /opt/other/redox/


# OTHER

if you go to other directories like recipes, cd back to /opt/other/redox/ after

commit often, small increments even if broken ( as WIP but note the challenges in the commit message )

Don't push to gitlab upstream, just to the origin fork!

If fixes work in the iso also apply them to build/aarch64/server-cranelift.qcow2 or use qcow2 directly, but create .bak !

blindly append all (semi) interesting finds and procedural insights to notes.md ( we may siff through them later )
whenever you encounter scripts that don't / do work or found some 'missing' files append that to notes.md 

Before and after each Bash command, give a short one-liner comment of what you are planning to achieve and what the result was. 

./snapshot.sh save feature_you_implemented if it works (or wip)
Always mention the working saved snapshot name in Git commits
