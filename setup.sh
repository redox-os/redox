#!/bin/bash

set -e

echo "Defaulting to rust nightly"
rustup override set nightly
echo "Downloading rust source"
rustup component add rust-src
if [ -z "$(which xargo)" ]
then
    echo "Installing xargo"
    cargo install -f xargo
fi

echo "Building libstd"
./cook.sh libstd unfetch
./cook.sh libstd fetch
./cook.sh libstd build
cp recipes/libstd/build/target/x86_64-unknown-redox/release/deps/*.rlib ~/.xargo/lib/rustlib/x86_64-unknown-redox/lib/

echo "cook.sh is ready to use"
