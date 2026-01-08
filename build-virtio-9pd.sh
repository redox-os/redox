#!/bin/bash
set -e

cd /opt/other/redox/recipes/core/base/source

SYSROOT=/opt/other/redox/build/aarch64/cranelift-sysroot
CRANELIFT_LIB=/opt/other/rustc_codegen_cranelift/dist/lib/librustc_codegen_cranelift.dylib
NIGHTLY=nightly-2026-01-02
TARGET=/opt/other/redox/tools/aarch64-unknown-redox-clif.json

export DYLD_LIBRARY_PATH=~/.rustup/toolchains/${NIGHTLY}-aarch64-apple-darwin/lib

RUSTFLAGS="-Zcodegen-backend=$CRANELIFT_LIB \
    -L $SYSROOT/lib \
    -Cpanic=abort \
    -Clink-arg=-z -Clink-arg=muldefs \
    -Clink-arg=-lunwind_stubs \
    -Clink-arg=$SYSROOT/lib/crt0.o \
    -Clink-arg=$SYSROOT/lib/crti.o \
    -Clink-arg=$SYSROOT/lib/crtn.o" \
cargo +$NIGHTLY build \
    --target $TARGET \
    -p virtio-9pd \
    --release \
    -Z build-std=std,core,alloc,panic_abort

echo "Build complete!"
ls -la target/aarch64-unknown-redox-clif/release/virtio-9pd 2>/dev/null || echo "Binary not found in expected location"
