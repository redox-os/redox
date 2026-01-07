#!/bin/bash
cd /opt/other/redox/recipes/core/base/source
RELIBC_DIR=/opt/other/redox/recipes/core/relibc/source/target/aarch64-unknown-redox-clif/release
export DYLD_LIBRARY_PATH="/Users/me/.rustup/toolchains/nightly-2026-01-02-aarch64-apple-darwin/lib"
export RUSTFLAGS="-Zcodegen-backend=/opt/other/rustc_codegen_cranelift/dist/lib/librustc_codegen_cranelift.dylib -L ${RELIBC_DIR} -Cpanic=abort -Clink-arg=-z -Clink-arg=muldefs -Clink-arg=-L${RELIBC_DIR} -Clink-arg=-lunwind_stubs -Clink-arg=${RELIBC_DIR}/crt0.o -Clink-arg=${RELIBC_DIR}/crt0_rust.o -Clink-arg=${RELIBC_DIR}/crti.o -Clink-arg=${RELIBC_DIR}/crtn.o"
cargo +nightly-2026-01-02 build --target aarch64-unknown-redox-clif.json --release -Z build-std=std,core,alloc,panic_abort -p ipcd
