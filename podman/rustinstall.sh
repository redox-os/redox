#/usr/bin/env bash

# This script install the Rust toolchain and the build system dependencies
# in Podman after the image has been built

curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain stable

SCCACHE_PATH=https://github.com/mozilla/sccache/releases/download/v0.10.0/sccache-v0.10.0-$(uname -m)-unknown-linux-musl.tar.gz
curl -sSL $SCCACHE_PATH | tar -xz -C ~/.cargo/bin --strip-components=1 --wildcards '*/sccache'
export RUSTC_WRAPPER=sccache

cargo +stable install --force --version 0.1.1 cargo-config
cargo +stable install --force --version 1.16.0 just
cargo +stable install --force --version 0.27.0 cbindgen
