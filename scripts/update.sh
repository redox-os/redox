#!/usr/bin/env bash

# Update Ubuntu/Debian-based systems
sudo apt update || true
sudo apt upgrade -y || true
# Update the Rust toolchain
rustup update || true
# Update the build system source and submodules
make pull
# Update the relibc folder timestamp
touch relibc
# Update relibc
make prefix
# Update recipes
make rebuild