#!/bin/bash
# Build virtio-netd for aarch64 using Cranelift-compiled relibc
set -e

REDOX_DIR="$(cd "$(dirname "$0")" && pwd)"
BASE_SOURCE="$REDOX_DIR/recipes/core/base/source"
RELIBC_LIB="$REDOX_DIR/recipes/core/relibc/source/target/aarch64-unknown-redox-clif/release/librelibc.a"
LIBDIR="/tmp/redox-aarch64-libs"

# Check relibc exists
if [[ ! -f "$RELIBC_LIB" ]]; then
    echo "Error: relibc not found at $RELIBC_LIB"
    echo "Build it first with build-relibc-aarch64.sh"
    exit 1
fi

# Set up library directory
mkdir -p "$LIBDIR"
cp "$RELIBC_LIB" "$LIBDIR/libc.a"
ar rcs "$LIBDIR/libgcc_eh.a"  # Empty stub

echo "Building virtio-netd for aarch64..."
cd "$BASE_SOURCE"

export CARGO_TARGET_AARCH64_UNKNOWN_REDOX_LINKER=rust-lld
export RUSTFLAGS="-L $LIBDIR -Z unstable-options -C panic=abort"

cargo +nightly build \
    -p virtio-netd \
    --target aarch64-unknown-redox \
    --release \
    -Z build-std=std,core,alloc,panic_abort

echo ""
echo "Build complete!"
ls -la "$BASE_SOURCE/target/aarch64-unknown-redox/release/virtio-netd"
