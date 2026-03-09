#!/usr/bin/env bash
# This must be run outside podman build so the build/podman volume mount to /root contains all home folder changes
set -ex

echo "Installing rust..."
curl "https://sh.rustup.rs" -sSf | sh -s -- -y --default-toolchain stable --profile minimal

echo "Downloading sccache..."
SCCACHE_URL="https://github.com/mozilla/sccache/releases/download/v0.10.0/sccache-v0.10.0-$(uname -m)-unknown-linux-musl.tar.gz"
wget -qO- --show-progress "${SCCACHE_URL}" | tar -xz -C ~/.cargo/bin --strip-components=1 --wildcards '*/sccache'

echo "Downloading just..."
JUST_URL="https://github.com/casey/just/releases/download/1.45.0/just-1.45.0-$(uname -m)-unknown-linux-musl.tar.gz"
wget -qO- --show-progress "${JUST_URL}" | tar -xz -C ~/.cargo/bin --wildcards 'just'

echo "Downloading cbindgen..."
CBINDGEN_NAME="$( [ "$(uname -m)" = "x86_64" ] && echo "ubuntu22.04" || echo "ubuntu22.04-aarch64" )"
CBINDGEN_URL="https://github.com/mozilla/cbindgen/releases/download/0.29.0/cbindgen-${CBINDGEN_NAME}"
wget -qO- --show-progress "${CBINDGEN_URL}" > ~/.cargo/bin/cbindgen
chmod +x ~/.cargo/bin/cbindgen
