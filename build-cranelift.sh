#!/bin/bash
# Build Redox OS components with Cranelift (pure Rust toolchain)
# Requires: nightly-2026-01-02 toolchain and Cranelift backend

set -e

NIGHTLY="nightly-2026-01-02"
CRANELIFT_LIB="/opt/other/rustc_codegen_cranelift/dist/lib/librustc_codegen_cranelift.dylib"
TOOLCHAIN_LIB="$HOME/.rustup/toolchains/${NIGHTLY}-aarch64-apple-darwin/lib"

export DYLD_LIBRARY_PATH="$TOOLCHAIN_LIB"

usage() {
    echo "Usage: $0 [kernel|relibc|drivers|all] [aarch64]"
    echo "  kernel  - Build kernel"
    echo "  relibc  - Build relibc"
    echo "  drivers - Build userspace drivers"
    echo "  all     - Build kernel + relibc + drivers (default)"
    exit 1
}

check_prerequisites() {
    if [[ ! -f "$CRANELIFT_LIB" ]]; then
        echo "Error: Cranelift backend not found at $CRANELIFT_LIB"
        echo "Build it with: cd /opt/other/rustc_codegen_cranelift && ./y.sh build"
        exit 1
    fi
}

build_kernel_aarch64() {
    echo "=== Building kernel for aarch64 ==="
    cd /opt/other/redox/recipes/core/kernel/source

    if [[ ! -f aarch64-redox-none.json ]]; then
        cat > aarch64-redox-none.json << 'EOF'
{
    "arch": "aarch64",
    "data-layout": "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128",
    "disable-redzone": true,
    "executables": true,
    "features": "+strict-align,+neon,+fp-armv8",
    "linker": "rust-lld",
    "linker-flavor": "ld.lld",
    "llvm-target": "aarch64-unknown-none",
    "max-atomic-width": 64,
    "panic-strategy": "abort",
    "relocation-model": "static",
    "target-pointer-width": "64"
}
EOF
    fi

    RUSTFLAGS="-Zcodegen-backend=$CRANELIFT_LIB" \
    cargo +$NIGHTLY build \
        --target aarch64-redox-none.json \
        --release \
        -Z build-std=core,alloc \
        -Zbuild-std-features=compiler-builtins-mem,compiler_builtins/no-f16-f128

    echo "✓ aarch64 kernel built"
    ls -lh target/aarch64-redox-none/release/kernel
}

build_relibc_aarch64() {
    echo "=== Building relibc for aarch64 ==="
    cd /opt/other/redox/recipes/core/relibc/source

    if [[ ! -f aarch64-unknown-redox-clif.json ]]; then
        cat > aarch64-unknown-redox-clif.json << 'EOF'
{
    "arch": "aarch64",
    "data-layout": "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128",
    "env": "relibc",
    "executables": true,
    "features": "+strict-align,+neon,+fp-armv8",
    "has-rpath": true,
    "linker": "rust-lld",
    "linker-flavor": "ld.lld",
    "llvm-target": "aarch64-unknown-redox",
    "max-atomic-width": 64,
    "os": "redox",
    "position-independent-executables": true,
    "relro-level": "full",
    "target-family": ["unix"],
    "target-pointer-width": "64"
}
EOF
    fi

    RUSTFLAGS="-Zcodegen-backend=$CRANELIFT_LIB" \
    cargo +$NIGHTLY build \
        --target aarch64-unknown-redox-clif.json \
        --release \
        -Z build-std=core,alloc \
        -Zbuild-std-features=compiler_builtins/no-f16-f128

    echo "✓ aarch64 relibc built"
    ls -lh target/aarch64-unknown-redox-clif/release/librelibc.a
}

build_drivers_aarch64() {
    echo "=== Building drivers for aarch64 ==="

    RELIBC_LIB="/opt/other/redox/recipes/core/relibc/source/target/aarch64-unknown-redox-clif/release/librelibc.a"
    if [[ ! -f "$RELIBC_LIB" ]]; then
        echo "Error: relibc not found. Run: $0 relibc"
        exit 1
    fi

    LIBDIR="/tmp/redox-aarch64-libs"
    mkdir -p "$LIBDIR"
    cp "$RELIBC_LIB" "$LIBDIR/libc.a"
    ar rcs "$LIBDIR/libgcc_eh.a"
    ar rcs "$LIBDIR/libgcc_s.a"

    # Create unwind stubs for backtrace symbols
    cat > /tmp/unwind_stubs.c << 'STUBEOF'
typedef void *_Unwind_Context;
void *_Unwind_GetTextRelBase(_Unwind_Context *ctx) { return (void*)0; }
void *_Unwind_GetDataRelBase(_Unwind_Context *ctx) { return (void*)0; }
void *_Unwind_FindEnclosingFunction(void *pc) { return (void*)0; }
unsigned long _Unwind_GetIP(_Unwind_Context *ctx) { return 0; }
unsigned long _Unwind_GetCFA(_Unwind_Context *ctx) { return 0; }
typedef int (*_Unwind_Trace_Fn)(_Unwind_Context *, void *);
int _Unwind_Backtrace(_Unwind_Trace_Fn trace, void *arg) { return 0; }
STUBEOF
    clang --target=aarch64-unknown-linux-gnu -c /tmp/unwind_stubs.c -o /tmp/unwind_stubs.o
    ar rcs "$LIBDIR/libunwind_stubs.a" /tmp/unwind_stubs.o

    cd /opt/other/redox/recipes/core/base/source

    # Custom target with max-atomic-width: 64 (Cranelift doesn't support 128-bit atomics)
    if [[ ! -f aarch64-unknown-redox-clif.json ]]; then
        cat > aarch64-unknown-redox-clif.json << 'EOF'
{
    "arch": "aarch64",
    "data-layout": "e-m:e-p270:32:32-p271:32:32-p272:64:64-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128-Fn32",
    "env": "relibc",
    "executables": true,
    "features": "+strict-align,+neon,+fp-armv8",
    "has-rpath": true,
    "linker": "rust-lld",
    "linker-flavor": "ld.lld",
    "llvm-target": "aarch64-unknown-redox",
    "max-atomic-width": 64,
    "os": "redox",
    "panic-strategy": "abort",
    "position-independent-executables": true,
    "relro-level": "full",
    "target-family": ["unix"],
    "target-pointer-width": 64
}
EOF
    fi

    # Use underscores for custom target env var
    export CARGO_TARGET_AARCH64_UNKNOWN_REDOX_CLIF_LINKER=rust-lld
    export RUSTFLAGS="-Zcodegen-backend=$CRANELIFT_LIB -L $LIBDIR -C panic=abort -C link-arg=-z -C link-arg=muldefs -C link-arg=-lunwind_stubs"

    DRIVERS=(virtio-netd virtio-blkd)

    for driver in "${DRIVERS[@]}"; do
        echo "Building $driver..."
        cargo +$NIGHTLY build \
            -p "$driver" \
            --target aarch64-unknown-redox-clif.json \
            --release \
            -Z build-std=std,core,alloc,panic_abort || echo "Warning: $driver failed"
    done

    echo ""
    echo "✓ aarch64 drivers:"
    ls -lh target/aarch64-unknown-redox-clif/release/virtio-* 2>/dev/null || echo "No drivers built"
}

COMPONENT="${1:-all}"

check_prerequisites

case "$COMPONENT" in
    kernel)  build_kernel_aarch64 ;;
    relibc)  build_relibc_aarch64 ;;
    drivers) build_drivers_aarch64 ;;
    all)
        build_kernel_aarch64
        build_relibc_aarch64
        build_drivers_aarch64
        ;;
    *) usage ;;
esac

echo ""
echo "=== Build complete ==="
