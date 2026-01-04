<p align="center">
<img alt="Redox" width="346" src="https://gitlab.redox-os.org/redox-os/assets/raw/master/logos/redox/logo.png">
</p>

<h2 align="center">100% Rust — No LLVM Required</h2>

<p align="center">
<b>Redox OS can now be compiled using a pure Rust toolchain.</b><br>
The kernel boots and relibc compiles using <a href="https://github.com/rust-lang/rustc_codegen_cranelift">Cranelift</a> — no C++ dependencies.
</p>

---

[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

## Quick Start — Pure Rust Build

Build Redox with only `cargo` and `rustup` — no LLVM, no C++ compiler needed:

```bash
# Install Rust nightly with Cranelift
rustup install nightly-2026-01-02
rustup component add rustc-codegen-cranelift-preview --toolchain nightly-2026-01-02

# Build the kernel
cd recipes/core/kernel/source
RUSTFLAGS="-Zcodegen-backend=cranelift -C relocation-model=static -C link-arg=-Tlinkers/x86_64.ld" \
cargo +nightly-2026-01-02 build --target x86_64-unknown-none --release \
  -Z build-std=core,alloc -Zbuild-std-features=compiler_builtins/no-f16-f128

# Build relibc (C library)
cd recipes/core/relibc/source
RUSTFLAGS="-Zcodegen-backend=cranelift" \
cargo +nightly-2026-01-02 build --target x86_64-unknown-redox --release \
  -Z build-std=core,alloc -Zbuild-std-features=compiler_builtins/no-f16-f128
```

**Note:** For advanced features (sym operands in global_asm, variadic C functions), use the enhanced Cranelift fork: [pannous/rustc_codegen_cranelift](https://github.com/pannous/rustc_codegen_cranelift)

### Run in QEMU

```bash
./run-cranelift-redox.sh
# Login: user (no password) or root/password
```

---

## About Redox

[Redox](https://www.redox-os.org) is an open-source operating system written in Rust, a language with focus on safety, efficiency and high performance. Redox uses a microkernel architecture, and aims to be reliable, secure, usable, correct, and free.

Redox _is not_ just a kernel, it's a **full-featured operating system**, providing components (file system, display server, core utilities, etc.) that together make up a functional and convenient operating system.

### Why Pure Rust Matters

| Traditional Toolchain | Pure Rust Toolchain |
|-----------------------|---------------------|
| LLVM: ~20 million lines of C++ | Cranelift: ~200K lines of Rust |
| Complex C++ build dependencies | Just `cargo` |
| Memory safety concerns in compiler | Memory-safe compiler |
| Difficult to audit | Auditable codebase |

## Links

- [Main Website](https://www.redox-os.org)
- [The Redox Book](https://doc.redox-os.org/book/)
- [Chat and Support](https://matrix.to/#/#redox-join:matrix.org)
- [Patreon](https://www.patreon.com/redox_os) | [Donate](https://redox-os.org/donate/) | [Merch](https://redox-os.creator-spring.com/)

### Documentation

- [Building Redox](https://doc.redox-os.org/book/podman-build.html) (traditional LLVM build)
- [Hardware Compatibility](https://doc.redox-os.org/book/hardware-support.html)
- [Running in VM](https://doc.redox-os.org/book/running-vm.html) | [Real Hardware](https://doc.redox-os.org/book/real-hardware.html)
- [Developer FAQ](https://doc.redox-os.org/book/developer-faq.html)
- [Contributing](CONTRIBUTING.md)

## Ecosystem

| Repository | Description |
|------------|-------------|
| [Kernel](https://gitlab.redox-os.org/redox-os/kernel) | Microkernel — now builds with Cranelift |
| [relibc](https://gitlab.redox-os.org/redox-os/relibc) | C library in Rust — now builds with Cranelift |
| [RedoxFS](https://gitlab.redox-os.org/redox-os/redoxfs) | Default filesystem |
| [Ion](https://gitlab.redox-os.org/redox-os/ion) | Default shell |
| [Orbital](https://gitlab.redox-os.org/redox-os/orbital) | Display server and window manager |
| [Base](https://gitlab.redox-os.org/redox-os/base) | Essential system components and drivers |

## What it looks like

See [Redox in Action](https://www.redox-os.org/screens/) for photos and videos.

<img alt="Redox" height="150" src="https://gitlab.redox-os.org/redox-os/website/-/raw/master/static/img/screenshot/orbital-visual.png">
<img alt="Redox" height="150" src="https://gitlab.redox-os.org/redox-os/website/-/raw/master/static/img/screenshot/cosmic-programs.png">
<img alt="Redox" height="150" src="https://gitlab.redox-os.org/redox-os/website/-/raw/master/static/img/screenshot/cosmic-term-screenfetch.png">

<img alt="Redox" height="150" src="https://gitlab.redox-os.org/redox-os/website/-/raw/master/static/img/screenshot/cosmic-edit-redox.png">
<img alt="Redox" height="150" src="https://gitlab.redox-os.org/redox-os/website/-/raw/master/static/img/screenshot/image-viewer.png">
<img alt="Redox" height="150" src="https://gitlab.redox-os.org/redox-os/assets/raw/master/screenshots/Boot.png">

---

## Cranelift Build Details

The Cranelift backend required several enhancements (available in [pannous/rustc_codegen_cranelift](https://github.com/pannous/rustc_codegen_cranelift)):

1. **`sym` operand support** — Required for kernel's `global_asm!` macros
2. **Variadic function definitions** — Required for relibc's C functions (`printf`, `syslog`, etc.)
3. **Unique wrapper symbols** — Fixed duplicate symbol linker errors

### Tested Configuration

- **Kernel**: Boots to login prompt in QEMU
- **relibc**: 16 MB library compiles successfully
- **Platform**: x86_64 (cross-compiled from macOS ARM)
- **Rust**: nightly-2026-01-02

See `.claude/CLAUDE.md` for detailed build notes and troubleshooting.
