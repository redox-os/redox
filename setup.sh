#!/bin/bash
set -e

echo "Downloading latest libc-artifacts"
git submodule update --init --remote libc-artifacts

echo "Defaulting to rust nightly"
rustup override set nightly
echo "Update rust nightly"
rustup update nightly
echo "Downloading rust source"
rustup component add rust-src
if [ -z "$(which xargo)" ]
then
    echo "Installing xargo"
    cargo install -f xargo
fi

echo "cook.sh is ready to use"
