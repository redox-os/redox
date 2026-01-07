#!/bin/bash
# Pure Rust build - no GCC/Clang needed
# Uses: Cranelift + libm (Rust) + rust-lld

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

# Use Rust's bundled LLVM tools instead of system GCC
RUST_TOOLCHAIN=${RUST_TOOLCHAIN:-nightly-2026-01-02}
RUST_SYSROOT=$(rustc +$RUST_TOOLCHAIN --print sysroot)

# LLVM tools from Rust
export AR="$RUST_SYSROOT/lib/rustlib/$(rustc +$RUST_TOOLCHAIN -vV | grep host | cut -d' ' -f2)/bin/llvm-ar"
export STRIP="$RUST_SYSROOT/lib/rustlib/$(rustc +$RUST_TOOLCHAIN -vV | grep host | cut -d' ' -f2)/bin/llvm-strip"
export OBJCOPY="$RUST_SYSROOT/lib/rustlib/$(rustc +$RUST_TOOLCHAIN -vV | grep host | cut -d' ' -f2)/bin/llvm-objcopy"

# If LLVM tools not in sysroot, try system llvm
if [ ! -f "$AR" ]; then
    AR=$(which llvm-ar 2>/dev/null || which ar)
    STRIP=$(which llvm-strip 2>/dev/null || which strip)
    OBJCOPY=$(which llvm-objcopy 2>/dev/null || which objcopy)
fi

# Cranelift backend
CRANELIFT_LIB="/opt/other/rustc_codegen_cranelift/dist/lib/librustc_codegen_cranelift.dylib"
if [ ! -f "$CRANELIFT_LIB" ]; then
    echo "Cranelift library not found at $CRANELIFT_LIB"
    echo "Build it first: cd /opt/other/rustc_codegen_cranelift && ./y.sh build"
    exit 1
fi

# Environment setup
export DYLD_LIBRARY_PATH="$RUST_SYSROOT/lib:$DYLD_LIBRARY_PATH"
export RUSTFLAGS="-Zcodegen-backend=$CRANELIFT_LIB"
export RUSTUP_TOOLCHAIN=$RUST_TOOLCHAIN

# rust-lld for linking (instead of gcc)
RUST_LLD="$RUST_SYSROOT/lib/rustlib/$(rustc +$RUST_TOOLCHAIN -vV | grep host | cut -d' ' -f2)/bin/rust-lld"
if [ -f "$RUST_LLD" ]; then
    export RUSTFLAGS="$RUSTFLAGS -C linker=$RUST_LLD"
fi

echo "=== Pure Rust Build Environment ==="
echo "Toolchain: $RUST_TOOLCHAIN"
echo "AR: $AR"
echo "STRIP: $STRIP"
echo "OBJCOPY: $OBJCOPY"
echo "RUSTFLAGS: $RUSTFLAGS"
echo ""

# Build targets
case "${1:-help}" in
    kernel)
        echo "Building kernel with Cranelift..."
        cd recipes/core/kernel/source
        cargo build --target x86_64-unknown-none --release \
            -Z build-std=core,alloc \
            -Zbuild-std-features=compiler-builtins-mem,compiler_builtins/no-f16-f128
        echo "Kernel built: target/x86_64-unknown-none/release/redox_kernel"
        ;;

    relibc-rust-only)
        echo "Building relibc Rust code only (no openlibm)..."
        cd recipes/core/relibc/source
        # Build just the Rust part, skip openlibm
        cargo build --target x86_64-unknown-redox --release \
            -Z build-std=core,alloc \
            -Zbuild-std-features=compiler_builtins/no-f16-f128
        echo "relibc built: target/x86_64-unknown-redox/release/librelibc.a"
        ;;

    shell)
        echo "Entering pure-Rust build shell..."
        echo "RUSTFLAGS and tools configured for CC-free builds"
        exec $SHELL
        ;;

    info)
        echo "Tools available:"
        echo "  AR: $(which $AR 2>/dev/null || echo 'not found')"
        echo "  STRIP: $(which $STRIP 2>/dev/null || echo 'not found')"
        echo "  rust-lld: $(which rust-lld 2>/dev/null || echo $RUST_LLD)"
        echo ""
        echo "To fully eliminate CC, relibc needs:"
        echo "  1. Replace openlibm (C) with libm crate wrappers"
        echo "  2. Use rust-lld for final libc.so linking"
        ;;

    *)
        echo "Usage: $0 {kernel|relibc-rust-only|shell|info}"
        echo ""
        echo "  kernel         - Build kernel with Cranelift"
        echo "  relibc-rust-only - Build relibc Rust code (no C math lib)"
        echo "  shell          - Enter shell with pure-Rust env"
        echo "  info           - Show available Rust tools"
        ;;
esac
