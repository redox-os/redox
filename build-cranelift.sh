#!/bin/bash
# Build Redox with Cranelift codegen backend
# This script configures the build to use Cranelift instead of LLVM

set -e

# Ensure GNU tools are available (macOS compatibility)
export PATH="/opt/homebrew/opt/m4/bin:/opt/homebrew/opt/autoconf/bin:/opt/homebrew/opt/automake/bin:$PATH"

# Ensure native compiler is used for host builds
export CC=cc
export CXX=c++

# Configuration
# NOTE: aarch64 has 128-bit atomics limitation in Cranelift, use x86_64 for now
ARCH="${ARCH:-x86_64}"
CONFIG="${CONFIG:-server}"
CRANELIFT_LIB="/opt/other/rustc_codegen_cranelift/dist/lib/librustc_codegen_cranelift.dylib"
NIGHTLY="nightly-2026-01-02"

# Verify Cranelift library exists
if [ ! -f "$CRANELIFT_LIB" ]; then
    echo "ERROR: Cranelift library not found at $CRANELIFT_LIB"
    echo "Build it with: cd /opt/other/rustc_codegen_cranelift && ./y.sh prepare && ./y.sh build --sysroot clif"
    exit 1
fi

# Get the Rust toolchain lib path
TOOLCHAIN_LIB="$HOME/.rustup/toolchains/${NIGHTLY}-aarch64-apple-darwin/lib"
if [ ! -d "$TOOLCHAIN_LIB" ]; then
    echo "ERROR: Rust toolchain not found at $TOOLCHAIN_LIB"
    echo "Install with: rustup toolchain install $NIGHTLY"
    exit 1
fi

echo "=== Building Redox with Cranelift ==="
echo "Architecture: $ARCH"
echo "Config: $CONFIG"
echo "Cranelift: $CRANELIFT_LIB"
echo "Toolchain: $NIGHTLY"
echo ""

# Export environment variables
export DYLD_LIBRARY_PATH="$TOOLCHAIN_LIB:$DYLD_LIBRARY_PATH"

# Cranelift RUSTFLAGS - note: build-std flags are handled by cookbook
export RUSTFLAGS="-Zcodegen-backend=$CRANELIFT_LIB"

# Use nightly toolchain
export RUSTUP_TOOLCHAIN="$NIGHTLY"

# Build configuration
export ARCH="$ARCH"
export CONFIG_NAME="$CONFIG"
export PODMAN_BUILD=0
# Use prebuilt toolchain to avoid host build issues on macOS
export PREFIX_BINARY=1

# Enable Cranelift in cookbook build scripts
export COOKBOOK_CRANELIFT=1

echo "=== Environment ==="
echo "DYLD_LIBRARY_PATH=$DYLD_LIBRARY_PATH"
echo "RUSTFLAGS=$RUSTFLAGS"
echo "RUSTUP_TOOLCHAIN=$RUSTUP_TOOLCHAIN"
echo "COOKBOOK_CRANELIFT=$COOKBOOK_CRANELIFT"
echo ""

# Rebuild cookbook to pick up Cranelift changes
rebuild_cookbook() {
    echo "=== Rebuilding cookbook with Cranelift support ==="
    # Build cookbook without Cranelift flags (it's a host tool)
    unset RUSTFLAGS
    unset RUSTUP_TOOLCHAIN
    cargo build --release
    # Restore Cranelift flags
    export RUSTFLAGS="-Zcodegen-backend=$CRANELIFT_LIB"
    export RUSTUP_TOOLCHAIN="$NIGHTLY"
}

# Parse command
CMD="${1:-build}"
shift || true

case "$CMD" in
    relibc)
        echo "=== Building relibc with Cranelift ==="
        rebuild_cookbook
        make cr.relibc "$@"
        ;;
    base)
        echo "=== Building base with Cranelift ==="
        rebuild_cookbook
        make r.base "$@"
        ;;
    kernel)
        echo "=== Building kernel with Cranelift ==="
        rebuild_cookbook
        make r.kernel "$@"
        ;;
    drivers)
        echo "=== Building drivers-initfs with Cranelift ==="
        rebuild_cookbook
        make r.drivers-initfs "$@"
        ;;
    build)
        echo "=== Full build with Cranelift ==="
        rebuild_cookbook
        make "$@"
        ;;
    repo)
        echo "=== Building repo with Cranelift ==="
        rebuild_cookbook
        make repo "$@"
        ;;
    clean)
        echo "=== Cleaning build ==="
        make clean "$@"
        ;;
    cookbook)
        echo "=== Rebuilding cookbook only ==="
        rebuild_cookbook
        ;;
    shell)
        echo "=== Starting shell with Cranelift environment ==="
        exec bash
        ;;
    *)
        echo "Usage: $0 [command]"
        echo ""
        echo "Commands:"
        echo "  relibc   - Build relibc"
        echo "  base     - Build base package"
        echo "  kernel   - Build kernel"
        echo "  drivers  - Build drivers-initfs"
        echo "  build    - Full build (default)"
        echo "  repo     - Build repo"
        echo "  clean    - Clean build"
        echo "  cookbook - Rebuild cookbook only"
        echo "  shell    - Start shell with Cranelift environment"
        exit 1
        ;;
esac
