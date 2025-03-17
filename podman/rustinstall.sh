#/usr/bin/env bash

# This script install the Rust toolchain and the build system dependencies
# in Podman after the image has been built

curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain stable
cargo +stable install --force --version 0.1.1 cargo-config
cargo +stable install --force --version 1.16.0 just
cargo +stable install --force --version 0.27.0 cbindgen
