#/usr/bin/env bash

# This script install the Rust toolchain and the build system dependencies
# in Podman after the image has been built

echo Installing rust...
curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain stable --profile minimal

echo Downloading sccache...
SCCACHE_URL=https://github.com/mozilla/sccache/releases/download/v0.10.0/sccache-v0.10.0-$(uname -m)-unknown-linux-musl.tar.gz
wget -qO- --show-progress $SCCACHE_URL | tar -xz -C ~/.cargo/bin --strip-components=1 --wildcards '*/sccache'

echo Downloading just...
JUST_URL=https://github.com/casey/just/releases/download/1.45.0/just-1.45.0-$(uname -m)-unknown-linux-musl.tar.gz
wget -qO- --show-progress $JUST_URL | tar -xz -C ~/.cargo/bin --wildcards 'just'

echo Downloading cbindgen...
CBINDGEN_NAME=$( [[ $(uname -m) = "x86_64" ]] && echo "ubuntu22.04" || echo "ubuntu22.04-aarch64" )
CBINDGEN_URL=https://github.com/mozilla/cbindgen/releases/download/0.29.0/cbindgen-$CBINDGEN_NAME
wget -qO- --show-progress $CBINDGEN_URL > ~/.cargo/bin/cbindgen
chmod +x ~/.cargo/bin/cbindgen
