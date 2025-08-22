#/usr/bin/env bash

# This script install the Rust toolchain and the build system dependencies
# in Podman after the image has been built

curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain stable

curl -sSLf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
cargo +stable binstall --no-confirm --force --version 0.10.0 sccache
cargo +stable binstall --no-confirm --force --version 1.42.4 just
cargo +stable binstall --no-confirm --force --version 0.29.0 cbindgen
cargo +stable install --force --version 0.1.1 cargo-config # TODO: Remove
