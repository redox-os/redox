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

# TODOs
⚠️ ATTENTION: cranelift-initfs/initfs/bin binaries are broken, rebuilding initfs crashes boot

### Ion Shell config file location (FIXED)
Ion does NOT use `~/.ionrc` - it uses XDG paths:
- **Config file**: `~/.config/ion/initrc` (not `.ionrc`!)
- Created the proper path in image

### Ion Shell "." (dot) command bug (IDENTIFIED)
`. script` fails with "Exec format error" but `source script` works.
**Root cause**: Ion doesn't register "." as a builtin alias for "source" in `src/lib/builtins/mod.rs`.
**Fix**: Add `.add(".", &builtin_source, SOURCE_DESC)` to `with_basic()` function.
**Workaround**: Use `source` instead of `.`
**Upstream**: https://gitlab.redox-os.org/redox-os/ion (needs PR)

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
