#/usr/bin/env bash

# Install Rust in Podman, after the image has been built

curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly
cargo install --force --version 0.1.1 cargo-config
cargo install --force --version 1.16.0 just
