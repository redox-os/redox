#!/bin/bash
# Build Redox OS components with Cranelift (pure Rust toolchain)
# Requires: nightly-2026-01-02 toolchain and Cranelift backend

set -e

NIGHTLY="nightly-2026-01-02"
CRANELIFT_LIB="/opt/other/rustc_codegen_cranelift/dist/lib/librustc_codegen_cranelift.dylib"
TOOLCHAIN_LIB="$HOME/.rustup/toolchains/${NIGHTLY}-aarch64-apple-darwin/lib"

export DYLD_LIBRARY_PATH="$TOOLCHAIN_LIB"

usage() {
    echo "Usage: $0 [kernel|relibc|all] [x86_64|aarch64|both]"
    echo "  kernel  - Build kernel only"
    echo "  relibc  - Build relibc only"
    echo "  all     - Build both (default)"
    echo ""
    echo "  x86_64  - Build for x86_64"
    echo "  aarch64 - Build for aarch64"
    echo "  both    - Build for both architectures (default)"
    exit 1
}

check_prerequisites() {
    if [[ ! -f "$CRANELIFT_LIB" ]]; then
        echo "Error: Cranelift backend not found at $CRANELIFT_LIB"
        echo "Build it with:"
        echo "  cd /opt/other/rustc_codegen_cranelift"
        echo "  ./y.sh prepare"
        echo "  ./y.sh build --sysroot clif"
        exit 1
    fi

    if ! rustup run $NIGHTLY rustc --version &>/dev/null; then
        echo "Error: Toolchain $NIGHTLY not installed"
        echo "Install with: rustup toolchain install $NIGHTLY"
        exit 1
    fi
}

build_kernel_x86_64() {
    echo "=== Building kernel for x86_64 ==="
    cd /opt/other/redox/recipes/core/kernel/source

    RUSTFLAGS="-Zcodegen-backend=$CRANELIFT_LIB \
               -C relocation-model=static \
               -C link-arg=-Tlinkers/x86_64.ld" \
    cargo +$NIGHTLY build \
        --target x86_64-unknown-none \
        --release \
        -Z build-std=core,alloc \
        -Zbuild-std-features=compiler-builtins-mem,compiler_builtins/no-f16-f128

    echo "✓ x86_64 kernel: target/x86_64-unknown-none/release/kernel"
    file target/x86_64-unknown-none/release/kernel
}

build_kernel_aarch64() {
    echo "=== Building kernel for aarch64 ==="
    cd /opt/other/redox/recipes/core/kernel/source

    # Create custom target if needed (max-atomic-width: 64 for Cranelift)
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

    echo "✓ aarch64 kernel: target/aarch64-redox-none/release/kernel"
    file target/aarch64-redox-none/release/kernel
}

build_relibc_x86_64() {
    echo "=== Building relibc for x86_64 ==="
    cd /opt/other/redox/recipes/core/relibc/source

    RUSTFLAGS="-Zcodegen-backend=$CRANELIFT_LIB" \
    cargo +$NIGHTLY build \
        --target x86_64-unknown-redox \
        --release \
        -Z build-std=core,alloc \
        -Zbuild-std-features=compiler_builtins/no-f16-f128

    echo "✓ x86_64 relibc: target/x86_64-unknown-redox/release/librelibc.a"
    ls -lh target/x86_64-unknown-redox/release/librelibc.a
}

build_relibc_aarch64() {
    echo "=== Building relibc for aarch64 ==="
    cd /opt/other/redox/recipes/core/relibc/source

    # Create custom target if needed
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

    echo "✓ aarch64 relibc: target/aarch64-unknown-redox-clif/release/librelibc.a"
    ls -lh target/aarch64-unknown-redox-clif/release/librelibc.a
}

# Parse arguments
COMPONENT="${1:-all}"
ARCH="${2:-both}"

check_prerequisites

case "$COMPONENT" in
    kernel)
        case "$ARCH" in
            x86_64) build_kernel_x86_64 ;;
            aarch64) build_kernel_aarch64 ;;
            both) build_kernel_x86_64; build_kernel_aarch64 ;;
            *) usage ;;
        esac
        ;;
    relibc)
        case "$ARCH" in
            x86_64) build_relibc_x86_64 ;;
            aarch64) build_relibc_aarch64 ;;
            both) build_relibc_x86_64; build_relibc_aarch64 ;;
            *) usage ;;
        esac
        ;;
    all)
        case "$ARCH" in
            x86_64) build_kernel_x86_64; build_relibc_x86_64 ;;
            aarch64) build_kernel_aarch64; build_relibc_aarch64 ;;
            both)
                build_kernel_x86_64
                build_kernel_aarch64
                build_relibc_x86_64
                build_relibc_aarch64
                ;;
            *) usage ;;
        esac
        ;;
    *) usage ;;
esac

echo ""
echo "=== Build complete ==="
